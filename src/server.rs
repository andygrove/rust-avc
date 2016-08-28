extern crate iron;
extern crate urlencoded;

use self::iron::prelude::*;
use self::iron::status;
use self::urlencoded::UrlEncodedQuery;

use std::collections::HashMap;

pub fn start_server() {

    fn process_request(req: &mut Request) -> IronResult<Response> {
        println!("URL: {}", req.url);

        match req.get_ref::<UrlEncodedQuery>() {
            Ok(ref hashmap) => {
                println!("Parsed GET request query string:\n {:?}", hashmap);

                match hashmap.get("action") {
                    Some(ref a) => {
                        match a[0].as_ref() {
                            "start" => Ok(Response::with((status::Ok, "Started!"))),
                            "stopped" => Ok(Response::with((status::Ok, "Stopped!"))),
                            _ => Ok(Response::with((status::Ok, "Huh?")))
                        }
                    },
                    None => Ok(Response::with((status::Ok, "Missing action")))
                }

            },
            Err(ref e) => {
                println!("{:?}", e);
                Ok(Response::with((status::Ok, "Hello World!")))
            }
        }
    }

    Iron::new(process_request).http("localhost:8080").unwrap();
}
