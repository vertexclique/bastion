use bastion::prelude::*;
use std::sync::Arc;

///
/// Prologue:
/// This example demonstrate a idiomatic way to implement the round robin
/// algorithm with bastion. We will use two groups of children, one will be
/// supervised by a dispatcher, named `Rounder`, the other one will call it.
/// Both groups will be supervised by the supervisor.
///
///         Bastion
///            |
///        Supervisor     
///       /          \
///    Caller      Rounder
///      |            |
///   Children     Children
///
/// 1. We want a group of children which will broadcast a message to a defined
/// target with a defined data.
/// 2. We want a group of children for receive and print the message.
/// 3. We want to use a dispatcher on the second group because we don't want to
/// target a particular child in the first to process the message.
///
fn main() {
    // We need bastion to run our program
    Bastion::init();
    // We create the supervisor and we add both groups on it
    Bastion::supervisor(caller_supervisor)
        .and_then(|_| Bastion::supervisor(rounder_supervisor))
        .expect("Couldn't create supervisor chain.");
    // We are starting the Bastion program now
    Bastion::start();
    // We are waiting until the Bastion has stopped or got killed
    Bastion::block_until_stopped();
}

fn caller_supervisor(supervisor: Supervisor) -> Supervisor {
    // We create a new children, it wrap the Bastion::Children method to add it on the supervisor
    supervisor.children(|children| caller_group(children))
}

fn rounder_supervisor(supervisor: Supervisor) -> Supervisor {
    // We are doing the same as above
    supervisor.children(|children| rounder_group(children))
}

fn caller_group(children: Children) -> Children {
    // We create the first group of children
    children
        // We create the function to exec
        .with_exec(move |ctx: BastionContext| {
            async move {
                let data_to_send: Vec<&str> =
                    vec!["data_1", "data_2", "data_3", "data_4", "data_5"];
                // We define the target which will receive the broadcasted message
                let target = BroadcastTarget::Group("Rounder".to_string());
                // We iterate on each data
                for data in data_to_send {
                    // We broadcast the message containing the data to the defined target
                    ctx.broadcast_message(target.clone(), data)
                }
                // We stop bastion here, because we don't have more data to send
                Bastion::stop();
                Ok(())
            }
        })
}

fn rounder_group(children: Children) -> Children {
    // We create the second group of children
    children
        // We want to have 5 children in this group
        .with_redundancy(5)
        // We want to have a disptacher named `Rounder`
        .with_dispatcher(Dispatcher::with_type(DispatcherType::Named(
            "Rounder".to_string(),
        )))
        // We create the function to exec when each children is called
        .with_exec(move |ctx: BastionContext| {
            async move {
                // We create a loop which run as long as the disptacher is alive
                loop {
                    msg! {
                        // We are waiting a msg
                        ctx.recv().await?,
                        // We define the behavior when we receive a new msg
                        raw_message: Arc<SignedMessage> => {
                            // We open the message
                            let message = Arc::try_unwrap(raw_message).unwrap();
                            msg! {
                                message,
                                // Because it's a broadcasted message we can use directly the ref
                                ref data: &str => {
                                    // And we print it
                                    println!("Received {}", data);
                                };
                                _: _ => ();
                            }
                        };
                        _: _ => ();
                    }
                }
            }
        })
}
