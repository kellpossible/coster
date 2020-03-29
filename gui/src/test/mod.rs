use tr::tr;

pub fn test_string() -> String {
    String::from(tr!("in the test module" => "Hello World, this is me!"))
}