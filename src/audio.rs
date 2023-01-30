use std::{sync::mpsc::{sync_channel}};

use jack::{ClientOptions, PortFlags, AudioIn};

pub fn get_audio() -> std::sync::mpsc::Receiver<Vec<f32>> {
    let options = ClientOptions::empty();
    let (client, _client_status) = jack::Client::new("led_matrix_rust", options)
        .expect("Failed to open JACK client");
    
    let output_ports = client.ports(None, None, PortFlags::IS_OUTPUT);
    dbg!(output_ports);

    let spec = AudioIn::default();
    let audio_in = client.register_port("led_matrix_robot_mouth", spec)
        .expect("Failed to make audio in port");

    let (tx, rx) = sync_channel(0);

    let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let slice_in = audio_in.as_slice(ps);
        tx.send(slice_in.to_vec()).expect("Failed to send slice to other thread");
        jack::Control::Continue
    };
    let process = jack::ClosureProcessHandler::new(process_callback);
    
    //keep the audio process alive
    std::thread::spawn(move || {
    
        // Activate the client, which starts the processing.
        let _active_client = client.activate_async((), process).unwrap();

        loop {}
    });

    rx
}