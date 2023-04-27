use std::io::{StdoutLock, Write};

use anyhow::Context;
use rustengan::{main_loop, Body, Message, Node};
use serde::{Deserialize, Serialize};

struct UniqueIdNode {
    id: usize,
    node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum UniqueIdPayload {
    Generate,
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
}
impl Node<UniqueIdPayload> for UniqueIdNode {
    fn from_init(init: rustengan::Init) -> Self {
        Self {
            id: 1,
            node_id: init.node_id,
        }
    }

    fn step(
        &mut self,
        input: Message<UniqueIdPayload>,
        stdout: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match input.body.payload {
            UniqueIdPayload::Generate => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: UniqueIdPayload::GenerateOk {
                            guid: format!("{}-{}", self.node_id, self.id),
                        },
                    },
                };

                serde_json::to_writer(&mut *stdout, &reply)
                    .context("serialize response to echo")?;

                stdout.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            }
            UniqueIdPayload::GenerateOk { .. } => {}
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<UniqueIdNode, UniqueIdPayload>()
}
