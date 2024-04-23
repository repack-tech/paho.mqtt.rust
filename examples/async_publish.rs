// paho-mqtt/examples/async_publish.rs
//
// Example application for Paho MQTT Rust library.
//
//! This is a simple MQTT asynchronous message publisher using the
//! Paho Rust library.
//!
//! This sample demonstrates:
//!   - Connecting to an MQTT broker
//!   - Publishing a message asynchronously
//

/*******************************************************************************
 * Copyright (c) 2017-2023 Frank Pagliughi <fpagliughi@mindspring.com>
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

use futures::executor::block_on;
use paho_mqtt as mqtt;
use std::{env, process};
use libc::ftok;

/////////////////////////////////////////////////////////////////////////////

fn main() {
    // Initialize the logger from the environment
    env_logger::init();

    // Command-line option(s)
    let host = env::args()
        .nth(1)
        .unwrap_or_else(|| "mqtt://localhost:1883".to_string());

    println!("Connecting to the MQTT server at '{}'", host);

    // Create the client
    let cli = mqtt::AsyncClient::new(host).unwrap_or_else(|err| {
        println!("Error creating the client: {}", err);
        process::exit(1);
    });

    cli.set_delivered_callback(|client, tok| {
        println!("CLBK {:?}", tok);
    });

    if let Err(err) = block_on(async {
        // Connect with default options and wait for it to complete or fail
        // The default is an MQTT v3.x connection.
        cli.connect(None).await?;

        // Create a message and publish it
        println!("Publishing a message on the topic 'test'");
        let msg = mqtt::Message::new("test", "Hello Rust MQTT world!", mqtt::QOS_1);
        let tok = cli.publish(msg);
        println!("PUB {:?}", tok.get_id());
        tok.await?;
        let msg = mqtt::Message::new("test", "Hello Rust MQTT world2!", mqtt::QOS_1);
        let tok = cli.publish(msg);
        println!("PUB {:?}", tok.get_id());
        tok.await?;

        // Disconnect from the broker
        println!("Disconnecting");
        cli.disconnect(None).await?;

        Ok::<(), mqtt::Error>(())
    }) {
        eprintln!("{}", err);
    }
}
