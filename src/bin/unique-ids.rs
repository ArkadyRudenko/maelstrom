use anyhow::Context;
use maelstrom::*;
use serde::{Deserialize, Serialize};
use std::io::StdoutLock;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Generate,
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
}

struct UniqueNode {
    id: usize,
    node: String,
}

impl Node<(), Payload> for UniqueNode {
    fn from_init(
        _state: (),
        init: Init,
        _tx: tokio::sync::mpsc::Sender<Event<Payload>>,
    ) -> anyhow::Result<Self> {
        Ok(UniqueNode {
            node: init.node_id,
            id: 1,
        })
    }
    fn step(&mut self, input: Event<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let Event::Message(input) = input else {
            panic!("got injected event when there`s no event injection")
        };

        let mut reply = input.into_reply(Some(&mut self.id));
        match reply.body.payload {
            Payload::Generate => {
                let mut guid = format!("{}-{}", self.node, self.id);
                guid.push_str(reply.body.id.unwrap().to_string().as_str());
                reply.body.payload = Payload::GenerateOk { guid };
                reply.send(output).context("reply to generate")?;
            }
            Payload::GenerateOk { .. } => {}
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    main_loop::<_, UniqueNode, _, _>(()).await
}
