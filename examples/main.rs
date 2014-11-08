extern crate pubsub;
use std::os;
use pubsub::{Msg, PubSub};

fn main() {
    let string: String;

    let connection_string = match os::getenv("REDIS_PORT") {
        Some(val) => { string = val.replace("tcp", "redis"); string.as_slice() },
        None => "redis://127.0.0.1/"
    };

    let mut psub = PubSub::new(connection_string);

    psub.subscribe("foo", foo_handler)
        .subscribe("bar", bar_handler)
        .listen();

    fn foo_handler(message: &Msg, _: &PubSub) {
        let channel = message.get_channel().unwrap_or("unknown".to_string());
        let payload = message.get_payload().unwrap_or("unknown".to_string());

        assert_eq!(channel, "foo".to_string())
        println!("OK! {} {}", channel, payload);
    }

    fn bar_handler(message: &Msg, pubsub: &PubSub) {
        let channel = message.get_channel().unwrap_or("unknown".to_string());
        let payload: String = message.get_payload().unwrap_or("unknown".to_string());

        assert_eq!(channel, "bar".to_string())
        println!("OK! {} {}", channel, payload);
        pubsub.publish("foo", payload.as_slice());
    }
}
