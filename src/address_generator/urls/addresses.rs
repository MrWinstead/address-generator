
use rocket::{Rocket, State};
use rocket::http::Status;
use rocket::response::Response;
use rocket_contrib::JSON;

use rand::{Rng, thread_rng};

use ::address_generator::ip_database::IPGeoDatabase;


#[derive(Debug)]
struct AddressHandlerState {
    new_addresses_key: String,
    ipgeo_database: IPGeoDatabase,
}

#[derive(Debug, Deserialize)]
struct UploadRequest {
    data: String,
    passkey: String,
}

#[allow(unmounted_route)]
#[get("/<country_code>")]
fn get_by_countrycode(country_code: &str) -> &str {
    println!("by_countrycode: {:?}", country_code);

    "10.0.0.1"
}


#[allow(unmounted_route)]
#[post("/update_source", data = "<source>")]
fn update_country_ip_source(source: JSON<UploadRequest>, handler_ctx: State<AddressHandlerState>)
        -> Response {
    println!("update: {:?} {:?}", source, handler_ctx);
    let mut resp = Response::new();
    let handler_ctx_inner = handler_ctx.inner();

    if source.passkey == handler_ctx_inner.new_addresses_key {
        resp.set_status(Status::Ok);
    }else {
        resp.set_status(Status::Unauthorized);
    }
    resp
}


pub fn mount_routes(instance: Rocket, base: &str) -> Rocket {
    instance.mount(
        base,
        routes![
            get_by_countrycode,
            update_country_ip_source,
        ]
    )
}

pub fn add_state(instance: Rocket) -> Rocket {
    let session_passkey = generate_session_passkey(20);
    println!("Session passkey is: {}", session_passkey);

    instance.manage(AddressHandlerState {
            new_addresses_key: session_passkey,
            ipgeo_database: IPGeoDatabase::new(),
        })
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
    fn get_by_country_code() {
        let mut rocket = ignite();
        rocket = mount_routes(rocket, "/addresses/");

        let mut req = MockRequest::new(Method::Get, "/addresses/ag/");
        let mut resp = req.dispatch_with(&rocket);

        assert_eq!(resp.status(), Status::Ok, "Did not return status OK");

        let body_str = resp.body().and_then(|b| b.into_string());
        match body_str {
            Some(x) => {
                println!("{:?}", x);
            }
            None => assert!(false, "No body returned")
        };
    }

    #[test]
    fn upload_new_source_bad_passkey() {
        let mut rocket = ignite();
        rocket = mount_routes(rocket, "/addresses/");

        let session_passkey = generate_session_passkey(20);
        rocket = rocket.manage(AddressHandlerState{new_addresses_key: session_passkey,
            ipgeo_database: IPGeoDatabase::new()});
        let mut req = MockRequest::new(Method::Post, "/addresses/update_source")
            .header(ContentType::JSON)
            .body(r#"{ "data": "value", "passkey": "asdf" }"#);
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
        rocket = rocket.manage(AddressHandlerState{new_addresses_key: session_passkey,
                                                    ipgeo_database: IPGeoDatabase::new()});

        let mut req = MockRequest::new(Method::Post, "/addresses/update_source")
            .header(ContentType::JSON)
            .body(format!("{{ \"data\": \"value\", \"passkey\": \"{}\" }}", session_passkey2));
        let resp = req.dispatch_with(&rocket);

        assert_eq!(resp.status(), Status::Ok, "Did not return status OK");
    }
}
