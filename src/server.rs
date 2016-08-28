extern crate iron;

use self::iron::prelude::*;
use self::iron::status;

fn main() {

    fn hello_world(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello World!")))
    }

    Iron::new(hello_world).http("localhost:80").unwrap();
    println!("On 80");
}