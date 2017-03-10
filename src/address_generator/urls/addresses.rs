
use rocket::{Rocket, State};
use rocket::http::Status;
use rocket::response::Response;
use std::io::Cursor;
use std::ops::DerefMut;
use std::sync::Mutex;


use ::address_generator::ip_database::IPGeoDatabase;


#[derive(Debug)]
struct AddressHandlerState {
    ipgeo_database: IPGeoDatabase,
}

type SharedAddressHandlerState = Mutex<AddressHandlerState>;

#[allow(unmounted_route)]
#[get("/")]
fn get_address<'a>(handler_ctx: State<SharedAddressHandlerState>) -> Response<'a> {
    let ah_mutex: &Mutex<AddressHandlerState> = &handler_ctx;
    let mut ah_state = ah_mutex.lock().unwrap();
    let ref mut ipg_db = ah_state.deref_mut().ipgeo_database;
    match ipg_db.get_random_country_code() {
        None => Response::build().status(Status::NoContent).finalize(),

        Some(country_code) => {
            match ipg_db.get_random_address(&country_code) {
                Err(_) => Response::build()
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
    let ah_mutex: &Mutex<AddressHandlerState> = &handler_ctx;
    let mut ah_state = ah_mutex.lock().unwrap();
    let ref mut ipg_db: IPGeoDatabase = ah_state.deref_mut().ipgeo_database;

    let address = ipg_db.get_random_address(&country_code.to_string());

    match address {
        Ok(addr) => Ok(format!("{}", addr)),
        Err(err) => {
            Err(Response::build()
                .status(Status::NotFound)
                .sized_body(Cursor::new(err.clone()))
                .finalize())
        }
    }
}

pub fn mount_routes(instance: Rocket, base: &str) -> Rocket {
    instance.mount(
        base,
        routes![
            get_address,
            get_by_countrycode,
        ]
    )
}

pub fn add_state(instance: Rocket, source_csv: &String) -> Rocket {
    let ipg_db = IPGeoDatabase::new(source_csv);
    match ipg_db {
        Ok(ipg_db) => {
            let wrapped_state = Mutex::new(
                AddressHandlerState {
                    ipgeo_database: ipg_db,
                });
            instance.manage(wrapped_state)
        },
        Err(why) => {
            println!("Could not interpret source csv: {}", why);
            instance
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use rocket::ignite;
    use rocket::testing::MockRequest;
    use rocket::http::{Status, Method, ContentType};
    use ::address_generator::ip_database::test::TEST_CSV;

    #[test]
    fn get_by_country_code_noprep() {
        let mut rocket = ignite();
        rocket = mount_routes(rocket, "/addresses/");
        rocket = add_state(rocket, &TEST_CSV.to_string());

        let mut req = MockRequest::new(Method::Get, "/addresses/ag/");
        let resp = req.dispatch_with(&rocket);

        assert_eq!(resp.status(), Status::NotFound, "Did not return status OK");
    }
}
