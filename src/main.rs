#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
// #[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use rocket::Data;
use rocket::request::LenientForm;
use rocket::request::Outcome;
use rocket::Request;
use rocket::request::FromRequest;
use rocket::http::{RawStr, Cookies, Cookie};

use rocket_contrib::json::Json;


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

// post form 模拟 put: curl -d "_method=PUT" -H "Content-Type: application/x-www-form-urlencoded" -X POST http://localhost:8000/requests/methods
// post form 模拟 put: curl -d "_method=PUT&a=1" -H "Content-Type: application/x-www-form-urlencoded" -X POST http://localhost:8000/requests/methods
// post form 模拟 put: curl -d "a=1&_method=PUT" -H "Content-Type: application/x-www-form-urlencoded" -X POST http://localhost:8000/requests/methods
#[put("/methods")]
fn methods() -> &'static str {
    "put"
}

// curl http://localhost:8000/requests/dynamic-paths/this.is.name
// curl http://localhost:8000/requests/dynamic-paths/%e4%b8%ad%e6%96%87
#[get("/dynamic-paths/<name>")]
fn dynamic_paths(name: &RawStr) -> String {
    format!("Hello, {}!", name.as_str())
}

// curl http://localhost:8000/requests/dynamic-paths/std-from-param/%e4%b8%ad%e6%96%87/1/true
#[get("/dynamic-paths/std-from-param/<name>/<age>/<cool>")]
fn std_from_param(name: String, age: u8, cool: bool) -> String {
    if cool {
        format!("You're a cool {} year old, {}!", age, name)
    } else {
        format!("{}, we need to talk about your coolness.", name)
    }
}

use std::path::PathBuf;

// curl http://localhost:8000/requests/dynamic-paths/multiple-segments/%e4%b8%ad%e6%96%87/1/true
// curl http://localhost:8000/requests/dynamic-paths/multiple-segments/../1/true # 404
#[get("/dynamic-paths/multiple-segments/<path..>")]
fn multiple_segments(path: PathBuf) -> String { 
    if let Some(p) = path.to_str() {
        String::from(p)
    } else {
        String::from("None")
    }
}

// curl http://localhost:8000/requests/forwarding/rank/1
#[get("/forwarding/rank/<id>")]
fn rank_default(id: usize) -> String { 
    String::from(format!("[rank] id={}", id))
}

#[get("/forwarding/rank/<id>", rank = 2)]
fn rank_2(id: isize) -> String { 
    String::from(format!("[rank_2] id={}", id))
}

// curl http://localhost:8000/requests/forwarding/rank/%e4%b8%ad%e6%96%87
#[get("/forwarding/rank/<id>", rank = 3)]
fn rank_3(id: &RawStr) -> String { 
    String::from(format!("[rank_3] id={}", id.url_decode().unwrap()))
}

// curl http://localhost:8000/requests/forwarding/option/1
// curl http://localhost:8000/requests/forwarding/option/string
#[get("/forwarding/option/<id>")]
fn option_param(id: Option<i32>) -> String {
    if let Some(v) = id {
        String::from(format!("param match id={}", v))
    } else {
        String::from("param not match")
    }
}

// curl http://localhost:8000/requests/forwarding/result/1
// curl http://localhost:8000/requests/forwarding/result/string
#[get("/forwarding/result/<id>")]
fn result_param(id: Result<i32, &RawStr>) -> String {
    if let Ok(v) = id {
        String::from(format!("param match id={}", v))
    } else if let Err(v) = id {
        String::from(format!("param not match id={}", v.url_decode().unwrap()))
    } else {
        String::from("dead code")
    }
}

// curl http://localhost:8000/requests/query-string/hello?wave # 404
// curl "http://localhost:8000/requests/query-string/hello?wave&name=xiaoming"
// curl "http://localhost:8000/requests/query-string/hello?wave=1&name=xiaoming" # 404
// curl "http://localhost:8000/requests/query-string/hello?name=xiaoming&wave"
// curl "http://localhost:8000/requests/query-string/hello?name=xiaoming&wave&age=10"
#[get("/query-string/hello?wave&<name>")]
fn query_string_hello(name: &RawStr) -> String {
    format!("Hello, {}!", name.as_str())
}

// curl "http://localhost:8000/requests/query-string/option?name=xiaoming"
// curl "http://localhost:8000/requests/query-string/option?wave&name=xiaoming"
// curl "http://localhost:8000/requests/query-string/option?wave=1&name=xiaoming"
// curl "http://localhost:8000/requests/query-string/option?name=xiaoming&wave"
// curl "http://localhost:8000/requests/query-string/option?name=xiaoming&wave=10"
#[get("/query-string/option?<wave>&<name>")]
#[allow(unused_variables)]
fn query_string_option(wave: Option<String>, name: &RawStr) -> String {
    format!("Hello, wave={:?}, name={}!", wave, name.as_str())
}

use rocket::request::Form;

#[derive(FromForm, Debug)]
struct User {
    name: String,
    account: usize,
}

// curl "http://localhost:8000/requests/query-string/multiple-segments?name=xiaoming&account=10&id=1"
#[get("/query-string/multiple-segments?<id>&<user..>")]
fn query_string_multiple_segments(id: usize, user: Form<User>) -> String{ 
    format!("id={}, user={:?}", id, user.0)
}

#[derive(UriDisplayQuery)]
struct ApiKey {
    key: String
}

impl<'a, 'r>  FromRequest<'a, 'r>  for ApiKey {
    type Error = String;
    
    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        if let Some(key) = request.headers().get_one("Authorization") {
            Outcome::Success(ApiKey { key: String::from(key) })
        } else {
            Outcome::Forward(())  // 没有的话继续匹配下一个
        }
    }
}

//  curl -H "Authorization: 123 " "http://localhost:8000/requests/request-guards/custom/api-key"
//  curl -H "authorization: 123 " "http://localhost:8000/requests/request-guards/custom/api-key"
#[get("/request-guards/custom/api-key")]
fn request_guards_custom_api_key1(key: ApiKey) -> String { 
    format!("api-key={}", key.key)
}

// curl "http://localhost:8000/requests/request-guards/custom/api-key"
#[get("/request-guards/custom/api-key", rank = 1)]
fn request_guards_custom_api_key2() -> String { 
    format!("No Authorization")
}

// curl -i -H "Cookie: message=messageCookieValue" "http://localhost:8000/requests/cookies" 
// curl -i -H "Cookie: k=v" "http://localhost:8000/requests/cookies"
// curl -i "http://localhost:8000/requests/cookies"
#[get("/cookies")]
fn request_cookies(mut cookies: Cookies) -> String {
    // 公有cookies
    cookies.add(Cookie::new("pub_key", "value"));
    cookies.remove(Cookie::named("pub_key2"));
    // 私有cookies 生产环境应该配置 secret_key 可以通过 openssl rand -base64 32 生成
    // 依赖 ring 库
    cookies.add_private(Cookie::new("pri_key", "value"));
    cookies.remove_private(Cookie::named("pri_key2"));
    cookies.get("message")
        .map(|value| format!("Message: {}", value))
        .unwrap_or("No message cookie".to_string())
}

#[derive(FromForm, Debug)]
struct Task {
    // 字段重命名
    #[form(field = "complete")]
    complete: bool,
    description: String,
}

// curl -d "complete=true&description=description" -H "Content-Type: application/x-www-form-urlencoded" -X POST http://localhost:8000/requests/body-data/form
// curl -d "complete=true&description=description" -X POST http://localhost:8000/requests/body-data/form
// curl -d "description=description" -X POST http://localhost:8000/requests/body-data/form
// curl -d "complete=1&description=description" -X POST http://localhost:8000/requests/body-data/form
// curl -d "a=1&complete=true&description=description" -X POST http://localhost:8000/requests/body-data/form
// 如果解析失败将返回 400 - Bad Request or 422 - Unprocessable Entity 
// 默认情况下会解析是严格模式：不允许多或少参数
#[post("/body-data/form", data = "<task>")]
fn body_data_form(task: Form<Task>) -> String { 
    format!("{:?}", task.0)
}

// curl -d "a=1&complete=true&description=description" -X POST http://localhost:8000/requests/body-data/lenient-form
// 宽容模式
#[post("/body-data/lenient-form", data = "<task>")]
fn body_data_lenient_form(task: LenientForm<Task>) -> String { 
    format!("{:?}", task.0)
}

// 引入 rocket_contrib serde serde_json serde_derive 依赖
#[derive(Deserialize, Debug)]
struct Task2 {
    description: String,
    complete: bool
}

// curl -d '{"complete": true, "description": "description"}' -X POST http://localhost:8000/requests/body-data/json
// curl -d '{"a": 1, "complete": true, "description": "description"}' -X POST http://localhost:8000/requests/body-data/json
// curl -d '{"complete1": true, "description": "description"}' -X POST http://localhost:8000/requests/body-data/json
// 默认是宽松模式
#[post("/body-data/json", data = "<task>")]
fn body_data_json(task: Json<Task2>) -> String { 
    format!("{:?}", task.0)
}

#[post("/upload", format = "plain", data = "<data>")]
fn request_upload(data: Data) -> std::io::Result<String> {
    // 需要防止DoS攻击
    data.stream_to_file("/tmp/upload.txt").map(|n| n.to_string())
}

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

use rocket::response::{content, status};

// curl -i http://localhost:8000/responses/status/accepted
#[get("/status/accepted")]
fn response_status_accepted() -> status::Accepted<String> {
    status::Accepted(Some("status::Accepted".to_string()))
}

// curl -i http://localhost:8000/responses/content/json
#[get("/content/json")]
fn response_content_json() -> content::Json<&'static str> {
    content::Json("{ 'hi': 'world' }")
}

#[derive(Serialize, Debug)]
struct Task3 {
    description: String,
    complete: bool
}

// curl http://localhost:8000/responses/content/rocket-contrib-json
#[get("/content/rocket-contrib-json")]
fn response_content_rocket_contrib_json() -> Json<Task3> { 
    Json(
        Task3 {
            description: "description".to_string(), 
            complete: true
        }
    )
}

// curl http://localhost:8000/typed-uris
#[get("/")]
fn typed_uris() -> String {
    let u1 = uri!(dynamic_paths: "test");
    let mut s = u1.to_string();
    s.push_str("\n");
    s.push_str(uri!(dynamic_paths: "中文").to_string().as_str());
    s.push_str("\n");
    s
}

fn main() {
    rocket::ignite()
        .register(catchers![not_found])
        .mount("/", routes![index])
        .mount("/requests", routes![methods, dynamic_paths])
        .mount("/requests", routes![std_from_param])
        .mount("/requests", routes![multiple_segments])
        .mount("/requests", routes![rank_default, rank_2, rank_3])
        .mount("/requests", routes![option_param, result_param])
        .mount("/requests", routes![query_string_hello])
        .mount("/requests", routes![query_string_option])
        .mount("/requests", routes![query_string_multiple_segments])
        .mount("/requests", routes![request_guards_custom_api_key1, request_guards_custom_api_key2])
        .mount("/requests", routes![request_cookies])
        .mount("/requests", routes![body_data_form, body_data_lenient_form])
        .mount("/requests", routes![body_data_json])
        .mount("/requests", routes![request_upload])
        .mount("/responses", routes![response_content_json, response_status_accepted])
        .mount("/responses", routes![response_content_rocket_contrib_json])
        .mount("/typed-uris", routes![typed_uris])
        .launch();
}
