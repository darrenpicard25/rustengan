use std::io::{StdoutLock, Write};

use anyhow::Context;
use rustengan::{main_loop, Body, Message, Node};
use serde::{Deserialize, Serialize};

struct EchoNode {
    id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum EchoPayload {
    Echo { echo: String },
    EchoOk { echo: String },
}
impl Node<EchoPayload> for EchoNode {
    fn from_init(_init: rustengan::Init) -> Self {
        Self { id: 1 }
    }

    fn step(&mut self, input: Message<EchoPayload>, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
            EchoPayload::Echo { echo } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: EchoPayload::EchoOk { echo },
                    },
                };

                serde_json::to_writer(&mut *stdout, &reply)
                    .context("serialize response to echo")?;

                stdout.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            }
            EchoPayload::EchoOk { .. } => {}
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<EchoNode, EchoPayload>()
}
