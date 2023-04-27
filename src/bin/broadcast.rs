use std::{
    collections::HashMap,
    io::{StdoutLock, Write},
};

use anyhow::Context;
use rustengan::{main_loop, Body, Message, Node};
use serde::{Deserialize, Serialize};

struct BroadcastNode {
    id: usize,
    messages: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum BroadcastPayload {
    Broadcast {
        message: i32,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<i32>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

impl BroadcastNode {
    fn construct_reply(
        &self,
        input: Message<BroadcastPayload>,
        payload: BroadcastPayload,
    ) -> Message<BroadcastPayload> {
        Message {
            src: input.dest,
            dest: input.src,
            body: Body {
                id: Some(self.id),
                in_reply_to: input.body.id,
                payload,
            },
        }
    }
}

impl Node<BroadcastPayload> for BroadcastNode {
    fn from_init(_init: rustengan::Init) -> Self {
        Self {
            id: 1,
            messages: Vec::new(),
        }
    }

    fn step(
        &mut self,
        input: Message<BroadcastPayload>,
        stdout: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match input.body.payload {
            BroadcastPayload::Broadcast { message } => {
                self.messages.push(message);
                let reply = self.construct_reply(input, BroadcastPayload::BroadcastOk);

                serde_json::to_writer(&mut *stdout, &reply)
                    .context("serialize response to broadcast")?;

                stdout.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            }
            BroadcastPayload::BroadcastOk { .. } => {}
            BroadcastPayload::Read => {
                let reply = self.construct_reply(
                    input,
                    BroadcastPayload::ReadOk {
                        messages: self.messages.clone(),
                    },
                );

                serde_json::to_writer(&mut *stdout, &reply)
                    .context("serialize response to read")?;

                stdout.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            }
            BroadcastPayload::ReadOk { .. } => {}
            BroadcastPayload::Topology { .. } => {
                let reply = self.construct_reply(input, BroadcastPayload::TopologyOk);

                serde_json::to_writer(&mut *stdout, &reply)
                    .context("serialize response to read")?;

                stdout.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            }
            BroadcastPayload::TopologyOk => {}
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<BroadcastNode, BroadcastPayload>()
}
