use midir::{MidiInput, Ignore};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new("midi_reader_input")?;
    midi_in.ignore(Ignore::None);


    // Loop until a MIDI input port is available
    let mut in_port = None;
    while in_port.is_none() {
        let ports = midi_in.ports();
        if !ports.is_empty() {
            in_port = Some(ports[0].clone()); // Assuming we want the first available port
        } else {
            println!("No MIDI input ports available. Waiting for connection...");
            std::thread::sleep(std::time::Duration::from_secs(5)); // Wait for 5 seconds before checking again
        }
    }

    // Now that we have a MIDI input port, continue with the rest of the program
    let in_port = in_port.unwrap(); // Safe to unwrap here due to the loop's logic
    println!("Listening on {}", midi_in.port_name(&in_port)?);

    let _conn_in = midi_in.connect(&in_port, "midi_reader", move |_, message, _| {
        println!("{:?}", message);

    }, ())?;

    std::thread::park();

    Ok(())
}
