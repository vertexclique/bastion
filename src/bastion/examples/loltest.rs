use bastion::prelude::*;

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

///
/// Prologue:
/// The most classic of all examples and especially the essential hello, world!
///
fn main() {
    // We need bastion to run our program
    Bastion::init();
    Bastion::start();

    #[derive(Default)]
    struct State {
        can_process_message: AtomicBool,
        messages_to_process: Arc<Vec<&'static str>>,
    }

    let my_state = State::default();
    // We are creating a group of children
    let workers = Bastion::children(|children| {
        // We are creating the function to exec
        children.with_context().with_exec(|ctx: BastionContext| {
            async move {
                // We are defining a behavior when a msg is received
                msg! {
                    // We are waiting a msg
                    ctx.recv().await?,
                    // We are catching a msg
                    msg: &'static str =!> {
                        if msg == "signal that allows me to process messages" {
                            my_state.can_process_messages.store(true, Ordering::SeqCst);
                            
                            let mut messages_to_process = my_state.messages_to_process.lock().unwrap();
                            for m in messages_to_process.iter() {
                                process_message(m);
                            }
                            messages_to_process.clear();
                        }

                        if my_state.can_process_messages.load(Ordering::SeqCst) == true {
                            process_message(msg);
                        } else {
                            my_state.messages_to_process.lock().push(msg);
                        }
                    };
                    _: _ => ();
                }
                Ok(())
            }
        })
    })
    .expect("Couldn't create the children group.");
    // We are creating the asker
    let asker = async {
        // We are getting the first (and only) worker
        let answer = workers.elems()[0]
            .ask_anonymously("hello, world!")
            .expect("Couldn't send the message.");
        // We are waiting for the asnwer
        answer.await.expect("couldn't receive answer");
    };
    // We are running the asker in the current blocked thread
    run!(asker);
    // We are stopping bastion here
    Bastion::stop();
}
