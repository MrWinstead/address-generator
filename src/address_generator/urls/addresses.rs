
use rocket::{Rocket, State};
use rocket::http::Status;
use rocket::response::{Response, Body};
use rocket::request::{Form, FromForm};
use rand::{Rng, thread_rng};
use std::io::Cursor;
use std::ops::DerefMut;
use std::sync::Mutex;


use ::address_generator::ip_database::IPGeoDatabase;


#[derive(Debug)]
struct AddressHandlerState {
    new_addresses_key: String,
    ipgeo_database: IPGeoDatabase,
}

#[derive(Debug, FromForm)]
struct UploadRequest {
    data: String,
    passkey: String,
}

type SharedAddressHandlerState = Mutex<AddressHandlerState>;

#[allow(unmounted_route)]
#[get("/")]
fn get_address<'a>(handler_ctx: State<SharedAddressHandlerState>) -> Response<'a> {
    let mut ah_mutex: &Mutex<AddressHandlerState> = &handler_ctx;
    let mut ah_state = ah_mutex.lock().unwrap();
    let ref mut ipg_db = ah_state.deref_mut().ipgeo_database;
    match ipg_db.get_random_country_code() {
        None => Response::build().status(Status::NoContent).finalize(),

        Some(country_code) => {
            match ipg_db.get_random_address(&country_code) {
                Err(e) => Response::build()
                    .status(Status::InternalServerError).finalize(),

                Ok(addr) => Response::build()
                    .sized_body(Cursor::new(format!("{}", addr)))
                    .finalize()
            }
        }
    }
}

#[allow(unmounted_route)]
#[get("/<country_code>")]
fn get_by_countrycode<'a>(country_code: &str, handler_ctx: State<SharedAddressHandlerState>)
                        -> Result<String, Response<'a>> {
    println!("by_countrycode: {:?}", country_code);
    let mut ah_mutex: &Mutex<AddressHandlerState> = &handler_ctx;
    let mut ah_state = ah_mutex.lock().unwrap();
    let ref mut ipg_db: IPGeoDatabase = ah_state.deref_mut().ipgeo_database;

    let address = ipg_db.get_random_address(&country_code.to_string());

    match address {
        Ok(addr) => Ok(format!("{}", addr)),
        Err(err) => {
            println!("--> {:?}", err);
            Err(Response::build()
                .status(Status::NotFound)
                .sized_body(Cursor::new(err.clone()))
                .finalize())
        }
    }
}

#[allow(unmounted_route)]
#[post("/update_source", data = "<source_form>")]
fn update_country_ip_source<'a>(source_form: Form<UploadRequest>,
                                handler_ctx: State<SharedAddressHandlerState>)
                                -> Response<'a> {
    let source = source_form.get();
    let mut ah_mutex: &SharedAddressHandlerState = &handler_ctx;
    let mut ah_state_tmp = ah_mutex.lock().unwrap();
    let mut ah_state = ah_state_tmp.deref_mut();
    let new_addresses_key = &ah_state.new_addresses_key;
    let ref mut ipg_db: IPGeoDatabase = ah_state.ipgeo_database;

    let mut resp = Response::new();
    let handler_ctx_inner = handler_ctx.inner();

    if &source.passkey == new_addresses_key {
        let store_result = ipg_db.set_underlying_data(source.data.clone());
        match store_result {
            Ok(x) => resp.set_status(Status::Ok),
            Err(y) => resp.set_status(Status::InternalServerError),
        };
    }else {
        resp.set_status(Status::Unauthorized);
    }
    resp
}


pub fn mount_routes(instance: Rocket, base: &str) -> Rocket {
    instance.mount(
        base,
        routes![
            get_address,
            get_by_countrycode,
            update_country_ip_source,
        ]
    )
}

pub fn add_state(instance: Rocket) -> Rocket {
    let session_passkey = generate_session_passkey(20);
    println!("Session passkey is: {}", session_passkey);
    instance.manage(
        Mutex::new(
            AddressHandlerState {
                new_addresses_key: session_passkey,
                ipgeo_database: IPGeoDatabase::new(),
    }))
}


fn generate_session_passkey(length: usize) -> String {
    thread_rng()
        .gen_ascii_chars()
        .take(length)
        .collect::<String>()
}


#[cfg(test)]
mod test {
    use super::*;
    use rocket::ignite;
    use rocket::testing::MockRequest;
    use rocket::http::{Status, Method, ContentType};

    #[test]
    fn get_by_country_code_noprep() {
        let mut rocket = ignite();
        rocket = mount_routes(rocket, "/addresses/");
        rocket = add_state(rocket);

        let mut req = MockRequest::new(Method::Get, "/addresses/ag/");
        let resp = req.dispatch_with(&rocket);

        assert_eq!(resp.status(), Status::NotFound, "Did not return status OK");
    }

    #[test]
    fn upload_new_source_bad_passkey() {
        let mut rocket = ignite();
        rocket = mount_routes(rocket, "/addresses/");
        rocket = add_state(rocket);

        let session_passkey = generate_session_passkey(20);
        rocket = rocket.manage(AddressHandlerState{new_addresses_key: session_passkey,
            ipgeo_database: IPGeoDatabase::new()});
        let mut req = MockRequest::new(Method::Post, "/addresses/update_source")
            .header(ContentType::Form)
            .body(r#"data=value&passkey=asdf"#);
        let resp = req.dispatch_with(&rocket);

        assert_eq!(resp.status(), Status::Unauthorized,
                    "Server allowed a request which should have failed");

    }

    #[test]
    fn upload_new_source() {
        let mut rocket = ignite();
        rocket = mount_routes(rocket, "/addresses/");

        let session_passkey = generate_session_passkey(20);
        let session_passkey2 = session_passkey.clone();
        rocket = rocket.manage(Mutex::new(AddressHandlerState{new_addresses_key: session_passkey,
                                                    ipgeo_database: IPGeoDatabase::new()}));

        let mut req = MockRequest::new(Method::Post, "/addresses/update_source")
            .header(ContentType::Form)
            .body(format!("data=value&passkey={}", session_passkey2));
        let resp = req.dispatch_with(&rocket);

        assert_eq!(resp.status(), Status::Ok, "Did not return status OK");
    }
}
