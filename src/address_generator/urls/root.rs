use rocket::response::status;
use rocket;

#[allow(unmounted_route)]
#[get("/")]
fn root() -> status::NoContent {
    status::NoContent
}


pub fn mount_routes(instance: rocket::Rocket, base: &str) -> rocket::Rocket {
    instance.mount(
        base,
        routes![
            root
        ]
    )
}

#[cfg(test)]
mod test {
    use super::rocket;
    use super::*;
    use rocket::testing::MockRequest;
    use rocket::http::{Status, Method};

    #[test]
    fn ensure_root_returns_nocontent() {
        let mut rocket = rocket::ignite();
        rocket = mount_routes(rocket, "/");

        let mut req = MockRequest::new(Method::Get, "/");
        let resp = req.dispatch_with(&rocket);

        assert_eq!(resp.status(), Status::NoContent);

    }

}
