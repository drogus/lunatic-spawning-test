use std::time::{Duration, Instant};

use lunatic::process::{sleep, Process};
use lunatic::{process, Config, Environment, Mailbox};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
enum Message {
    Spawn,
    Finish,
    GetCount(process::Process<usize>),
}

#[lunatic::main]
fn main(m: Mailbox<()>) {
    let mut config = Config::new(1000000000, Some(1000000000));
    config.allow_namespace("");
    let mut env = Environment::new(config).unwrap();
    let module = env.add_this_module().unwrap();

    module
        .spawn_link(m, |parent: Mailbox<()>| {
            let counter = process::spawn(counter_handle).unwrap();
            process::spawn_with(counter.clone(), display_handle).unwrap();
            for _ in 0..8 {
                process::spawn_with(counter.clone(), |counter, _: Mailbox<()>| {
                    for _ in 0..1000000 {
                        process::spawn_with(counter.clone(), handle).unwrap();
                    }
                })
                .unwrap();
            }
        })
        .unwrap();
    sleep(1000000);
}

fn handle(counter: Process<Message>, _: Mailbox<()>) {
    counter.send(Message::Spawn);
}

fn counter_handle(mailbox: Mailbox<Message>) {
    let mut count = 0;
    loop {
        match mailbox.receive().unwrap() {
            Message::Spawn => count += 1,
            Message::Finish => count -= 1,
            Message::GetCount(ps) => {
                ps.send(count);
            }
        }
    }
}

fn display_handle(counter: Process<Message>, mailbox: Mailbox<usize>) {
    let now = Instant::now();
    loop {
        let ps = process::this(&mailbox);
        counter.send(Message::GetCount(ps));
        let count = mailbox.receive().unwrap();
        let rate = (count as f64) / (now.elapsed().as_secs() as f64);
        println!("Spawn rate: {}ops", rate);
        println!("Handles count: {}", count);
        sleep(Duration::from_secs(1).as_millis() as u64);
    }
}
