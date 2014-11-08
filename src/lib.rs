extern crate redis;
pub use redis::Msg;
use std::io::Timer;
use std::time::Duration;
use std::collections::HashMap;

pub trait Handler: Send + Sync {
    fn call(&self, &Msg, &PubSub);
}

impl Handler for fn(&Msg, &PubSub) {
    fn call(&self, message: &Msg, pubsub: &PubSub) {
        (*self)(message, pubsub)
    }
}

impl Handler for Box<Handler + Send + Sync> {
    fn call(&self, message: &Msg, pubsub: &PubSub) {
        (**self).call(message, pubsub)
    }
}

pub struct PubSub {
    client: redis::Client,
    handlers: HashMap<String, Box<Handler>>
}

impl PubSub {
    pub fn new(connection_string: &str) -> PubSub {
        PubSub { client: redis::Client::open(connection_string).unwrap(), handlers: HashMap::new() }
    }

    pub fn subscribe<H: Handler>(&mut self, channel: &str, handler: H) -> &mut PubSub {
        self.handlers.insert(channel.to_string(), box handler as Box<Handler>);
        self
    }

    pub fn publish(&self, channel: &str, payload: &str) -> &PubSub {
        let con = self.client.get_connection();
        redis::cmd("PUBLISH").arg(channel).arg(payload.to_string()).execute(&con);
        self
    }

    pub fn listen(&self) {
        let mut pubsub = self.client.get_pubsub().unwrap();

        for (channel, _) in self.handlers.iter() {
            pubsub.subscribe(channel.as_slice()).unwrap();
        }

        let mut timer = Timer::new().unwrap();
        let periodic = timer.periodic(Duration::milliseconds(100));

        loop {
            periodic.recv();

            loop {
                let msg_result = pubsub.get_message();

                if msg_result.is_ok() {
                    let msg = msg_result.unwrap();
                    let channel = msg.get_channel().unwrap_or("default".to_string());

                    if self.handlers.contains_key(&channel) {
                        self.handlers[channel].call(&msg, self);
                    }

                    continue;
                }

                break
            }
        }
    }
}

