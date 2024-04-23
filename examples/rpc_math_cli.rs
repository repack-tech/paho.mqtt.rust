// paho-mqtt/examples/rpc_math_cli.rs
//
// This is a Paho MQTT v5 Rust sample application.
//
//!
//! It's an example of how to create a client for performing remote procedure
//! calls using MQTT with the 'response topic' and 'correlation data'
//! properties.
//!
//! The sample demonstrates:
//!  - Creating a dynamic RPC client for MQTT v5
//!  - Connecting to an MQTT v5 server/broker
//!  - Using MQTT v5 properties
//!  - Publishing RPC request messages
//!  - Using asynchronous tokens
//!  - Subscribing to reply topic
//

/*******************************************************************************
 * Copyright (c) 2019-2023 Frank Pagliughi <fpagliughi@mindspring.com>
 *
 * All rights reserved. This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License v2.0
 * and Eclipse Distribution License v1.0 which accompany this distribution.
 *
 * The Eclipse Public License is available at
 *    http://www.eclipse.org/legal/epl-v20.html
 * and the Eclipse Distribution License is available at
 *   http://www.eclipse.org/org/documents/edl-v10.php.
 *
 * Contributors:
 *    Frank Pagliughi - initial implementation and documentation
 *******************************************************************************/

#[macro_use]
extern crate paho_mqtt as mqtt;

use serde_json::json;
use std::{env, process};

/////////////////////////////////////////////////////////////////////////////

fn main() -> mqtt::Result<()> {
    // Initialize the logger from the environment
    env_logger::init();

    // We use the broker on this host.
    let host = "localhost";

    // Command-line option(s)
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() < 3 {
        println!("USAGE: rpc_math_cli <add|mult> <num1> <num2> [... numN]");
        process::exit(1);
    }

    const QOS: i32 = 1;

    const REQ_TOPIC_HDR: &str = "requests/math";
    const REP_TOPIC_HDR: &str = "replies/math";

    // Create a client to the specified host, no persistence
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .finalize();

    let cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|err| {
        eprintln!("Error creating the client: {}", err);
        process::exit(1);
    });

    // Initialize the consumer before connecting.
    // With a clean session/start, this order isn't important,
    // but it's still a good habit to start consuming first.
    let rx = cli.start_consuming();

    // Connect with default options for MQTT v5, (clean start)
    let conn_opts = mqtt::ConnectOptions::new_v5();

    // Connect and wait for it to complete or fail

    let rsp = cli.connect(conn_opts).wait().unwrap_or_else(|err| {
        eprintln!("Unable to connect: {:?}", err);
        process::exit(1);
    });

    // We get the assigned Client ID from the properties in the connection
    // response. The Client ID will help form a unique "reply to" topic
    // for us.

    let client_id = rsp
        .properties()
        .get_string(mqtt::PropertyCode::AssignedClientIdentifer)
        .unwrap_or_else(|| {
            eprintln!("Unable to retrieve Client ID");
            process::exit(1);
        });

    // We form a unique reply topic based on the Client ID,
    // and then subscribe to that topic.
    // (Be sure to subscribe *before* starting to send requests)
    let reply_topic = format!("{}/{}", REP_TOPIC_HDR, client_id);
    cli.subscribe(&reply_topic, QOS).wait()?;

    let corr_id = b"1";

    let props = mqtt::properties![
        mqtt::PropertyCode::ResponseTopic => reply_topic,
        mqtt::PropertyCode::CorrelationData => corr_id,
    ];

    // The request topic will be of the form:
    //     "requests/math/<operation>"
    // where we get <operation> ("add", "mult", etc) from the command line.

    let req_topic = format!("{}/{}", REQ_TOPIC_HDR, args[0]);

    // The payload is the JSON array of arguments for the operation.
    // These are the remaining arguments from the command line.

    let math_args: Vec<_> = args[1..]
        .iter()
        .map(|s| s.parse::<f64>())
        .filter_map(Result::ok)
        .collect();

    let payload = json!(math_args).to_string();

    // Create a message and publish it
    let msg = mqtt::MessageBuilder::new()
        .topic(req_topic)
        .payload(payload)
        .qos(QOS)
        .properties(props)
        .finalize();

    let tok = cli.publish(msg);

    if let Err(e) = tok.wait() {
        eprintln!("Error sending message: {:?}", e);
        cli.disconnect(None).wait().unwrap();
        process::exit(2);
    }

    // Wait for the reply and check the Correlation ID
    // Since we only sent one request, this should certainly be our reply!

    if let Some(msg) = rx.recv().unwrap() {
        let reply_corr_id = msg
            .properties()
            .get_binary(mqtt::PropertyCode::CorrelationData)
            .unwrap();

        if reply_corr_id == corr_id {
            let ret: f64 = serde_json::from_str(&msg.payload_str()).unwrap();
            println!("{}", ret);
        }
        else {
            eprintln!("Unknown response for {:?}", reply_corr_id);
        }
    }
    else {
        eprintln!("Error receiving reply.");
    }

    // Disconnect from the broker
    cli.disconnect(None).wait()?;
    Ok(())
}
