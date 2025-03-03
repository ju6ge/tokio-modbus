// SPDX-FileCopyrightText: Copyright (c) 2017-2022 slowtec GmbH <post@slowtec.de>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! RTU server example

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use futures::future;
    use std::{thread, time::Duration};

    use tokio_modbus::prelude::*;
    use tokio_modbus::server::{self, Service};

    struct MbServer;

    impl Service for MbServer {
        type Request = Request;
        type Response = Response;
        type Error = std::io::Error;
        type Future = future::Ready<Result<Self::Response, Self::Error>>;

        fn call(&self, req: Self::Request) -> Self::Future {
            match req {
                Request::ReadInputRegisters(_addr, cnt) => {
                    let mut registers = vec![0; cnt.into()];
                    registers[2] = 0x77;
                    future::ready(Ok(Response::ReadInputRegisters(registers)))
                }
                _ => unimplemented!(),
            }
        }
    }

    let builder = tokio_serial::new("/dev/ttyUSB0", 19200);
    let server_serial = tokio_serial::SerialStream::open(&builder).unwrap();

    println!("Starting up server...");
    let _server = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let server = server::rtu::Server::new(server_serial);
        rt.block_on(async {
            server.serve_forever(|| Ok(MbServer)).await;
        });
    });

    // Give the server some time for stating up
    thread::sleep(Duration::from_secs(1));

    println!("Connecting client...");
    let client_serial = tokio_serial::SerialStream::open(&builder).unwrap();
    let mut ctx = rtu::connect(client_serial).await?;
    println!("Reading input registers...");
    let rsp = ctx.read_input_registers(0x00, 7).await?;
    println!("The result is '{:#x?}'", rsp); // The result is '[0x0,0x0,0x77,0x0,0x0,0x0,0x0,]'

    Ok(())
}
