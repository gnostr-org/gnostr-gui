use crate::error::{Error, ErrorKind};
use crate::globals::GLOBALS;
use crate::people::PersonList;
use bech32::FromBase32;
use nostr_types::{
    Event, EventAddr, EventKind, Id, NostrBech32, NostrUrl, PrivateKey, PublicKey, RelayUrl,
    UncheckedUrl, Unixtime,
};
use std::env;
use tokio::runtime::Runtime;
use zeroize::Zeroize;

pub fn handle_command(mut args: env::Args, runtime: &Runtime) -> Result<bool, Error> {
    let _ = args.next(); // program name
    let command = args.next().unwrap(); // must be there or we would not have been called

    #[cfg(debug_assertions)]
    println!("\n*** gnostr-gui is running in command mode ***");
    #[cfg(debug_assertions)]
    println!("*** COMMAND = {} ***\n", command);

    match &*command {
        "bech32_decode" => bech32_decode(args)?,
        "bech32_encode_event_addr" => bech32_encode_event_addr(args)?,
        "decrypt" => decrypt(args)?,
        "events_of_kind" => events_of_kind(args)?,
        "events_of_pubkey_and_kind" => events_of_pubkey_and_kind(args)?,
        "giftwrap_ids" => giftwrap_ids()?,
        "help" => help()?,
        "--help" => help()?,
        "-h" => help()?,
        "login" => {
            login()?;
            return Ok(false);
        }
        "print_event" => print_event(args)?,
        "print_followed" => print_followed()?,
        "print_muted" => print_muted()?,
        "print_person_relays" => print_person_relays(args)?,
        "print_relay" => print_relay(args)?,
        "print_relays" => print_relays()?,
        "rebuild_indices" => rebuild_indices()?,
        "reprocess_recent" => reprocess_recent(runtime)?,
        "ungiftwrap" => ungiftwrap(args)?,
        "verify" => verify(args)?,
        "verify_json" => verify_json(args)?,
        other => println!("Unknown command {}", other),
    }

    Ok(true)
}

pub fn help() -> Result<(), Error> {
    println!("gnostr-gui bech32_decode <bech32string>");
    println!("    decode the bech32 string.");
    println!("gnostr-gui bech32_encode_event_addr <kind> <pubkeyhex> <d> [<relayurl>, ...]");
    println!("    encode an event address (parameterized replaceable event link).");
    println!("gnostr-gui decrypt <pubkeyhex> <ciphertext> <padded?>");
    println!("    decrypt the ciphertext from the pubkeyhex. padded=0 to not expect padding.");
    println!("gnostr-gui events_of_kind <kind>");
    println!("    print IDs of all events of kind=<kind>");
    println!("gnostr-gui events_of_pubkey_and_kind <pubkeyhex> <kind>");
    println!("    print IDs of all events from <pubkeyhex> of kind=<kind>");
    println!("gnostr-gui giftwrap_ids");
    println!("    List the IDs of all giftwrap events you are tagged on");
    println!("gnostr-gui help");
    println!("    show this list");
    println!("gnostr-gui login");
    println!("    login on the command line before starting the gnostr-gui GUI");
    println!("gnostr-gui print_event <idhex>");
    println!("    print the event (in JSON) from the database that has the given id");
    println!("gnostr-gui print_followed");
    println!("    print every pubkey that is followed");
    println!("gnostr-gui print_muted");
    println!("    print every pubkey that is muted");
    println!("gnostr-gui print_person_relays <pubkeyhex>");
    println!("    print all the person-relay records for the given person");
    println!("gnostr-gui print_relay <url>");
    println!("    print the relay record");
    println!("gnostr-gui print_relays");
    println!("    print all the relay records");
    println!("gnostr-gui rebuild_indices");
    println!("    Rebuild all event-related indices");
    println!("gnostr-gui reprocess_recent");
    println!("    Reprocess events that came during the last 24 hours");
    println!("gnostr-gui ungiftwrap <idhex>");
    println!("    Unwrap the giftwrap event with the given ID and print the rumor (in JSON)");
    println!("gnostr-gui verify <idhex>");
    println!("    Verify if the given event signature is valid");
    println!("gnostr-gui verify_json <event_json>");
    println!("    Verify if the passed in event JSON's signature is valid");

    Ok(())
}

pub fn bech32_decode(mut args: env::Args) -> Result<(), Error> {
    let mut param = match args.next() {
        Some(s) => s,
        None => {
            return Err(ErrorKind::Usage(
                "Missing bech32string parameter".to_string(),
                "bech32_decode <bech32string>".to_string(),
            )
            .into())
        }
    };

    // Also work if prefixed with 'nostr:'
    if let Some(nurl) = NostrUrl::try_from_string(&param) {
        param = format!("{}", nurl.0);
    }

    if let Some(nb32) = NostrBech32::try_from_string(&param) {
        match nb32 {
            NostrBech32::EventAddr(ea) => {
                println!("Event Address:");
                println!("  d={}", ea.d);
                println!(
                    "  relays={}",
                    ea.relays
                        .iter()
                        .map(|r| r.as_str().to_owned())
                        .collect::<Vec<String>>()
                        .join(", ")
                );
                println!("  kind={}", Into::<u32>::into(ea.kind));
                println!("  author={}", ea.author.as_hex_string());
            }
            NostrBech32::EventPointer(ep) => {
                println!("Event Pointer:");
                println!("  id={}", ep.id.as_hex_string());
                println!(
                    "  relays={}",
                    ep.relays
                        .iter()
                        .map(|r| r.as_str().to_owned())
                        .collect::<Vec<String>>()
                        .join(", ")
                );
                if let Some(kind) = ep.kind {
                    println!("  kind={}", Into::<u32>::into(kind));
                }
                if let Some(author) = ep.author {
                    println!("  author={}", author.as_hex_string());
                }
            }
            NostrBech32::Id(id) => {
                println!("Id: {}", id.as_hex_string());
            }
            NostrBech32::Profile(profile) => {
                println!("Profile:");
                println!("  pubkey: {}", profile.pubkey.as_hex_string());
                println!(
                    "  relays={}",
                    profile
                        .relays
                        .iter()
                        .map(|r| r.as_str().to_owned())
                        .collect::<Vec<String>>()
                        .join(", ")
                );
            }
            NostrBech32::Pubkey(pubkey) => {
                println!("Pubkey: {}", pubkey.as_hex_string());
            }
            NostrBech32::Relay(url) => {
                println!("Relay URL: {}", url.0);
            }
        }
    } else if let Ok(mut key) = PrivateKey::try_from_bech32_string(&param) {
        println!("Private Key: {}", key.as_hex_string());
    } else {
        let data = bech32::decode(&param).unwrap();
        println!("DATA.0 = {}", data.0);
        let decoded = Vec::<u8>::from_base32(&data.1).unwrap();
        println!("DATA.1 = {}", String::from_utf8_lossy(&decoded));
    }

    Ok(())
}

pub fn bech32_encode_event_addr(mut args: env::Args) -> Result<(), Error> {
    let kind: EventKind = match args.next() {
        Some(integer) => integer.parse::<u32>()?.into(),
        None => {
            return Err(ErrorKind::Usage(
                "Missing kind parameter".to_string(),
                "bech32_encode_event_addr <kind> <pubkeyhex> <d> [<relayurl>, ...]".to_owned(),
            )
            .into())
        }
    };

    let pubkey = match args.next() {
        Some(hex) => PublicKey::try_from_hex_string(&hex, true)?,
        None => {
            return Err(ErrorKind::Usage(
                "Missing pubkeyhex parameter".to_string(),
                "bech32_encode_event_addr <kind> <pubkeyhex> <d> [<relayurl>, ...]".to_owned(),
            )
            .into())
        }
    };

    let d = match args.next() {
        Some(d) => d,
        None => {
            return Err(ErrorKind::Usage(
                "Missing d parameter".to_string(),
                "bech32_encode_event_addr <kind> <pubkeyhex> <d> [<relayurl>, ...]".to_owned(),
            )
            .into())
        }
    };

    let mut urls: Vec<UncheckedUrl> = vec![];

    for s in args {
        urls.push(UncheckedUrl::from_string(s));
    }

    let ea = EventAddr {
        d,
        relays: urls,
        kind,
        author: pubkey,
    };

    println!("{}", ea.as_bech32_string());

    Ok(())
}

pub fn decrypt(mut args: env::Args) -> Result<(), Error> {
    let pubkey = match args.next() {
        Some(hex) => PublicKey::try_from_hex_string(&hex, true)?,
        None => {
            return Err(ErrorKind::Usage(
                "Missing pubkeyhex parameter".to_string(),
                "decrypt <pubkeyhex> <ciphertext> <padded?>".to_owned(),
            )
            .into())
        }
    };

    let ciphertext = match args.next() {
        Some(text) => text,
        None => {
            return Err(ErrorKind::Usage(
                "Missing ciphertext parameter".to_string(),
                "decrypt <pubkeyhex> <ciphertext> <padded?>".to_owned(),
            )
            .into())
        }
    };

    let padded = match args.next() {
        Some(padded) => padded == "1",
        None => {
            return Err(ErrorKind::Usage(
                "Missing padded parameter".to_string(),
                "decrypt <pubkeyhex> <ciphertext> <padded?>".to_owned(),
            )
            .into())
        }
    };

    login()?;

    let plaintext_bytes = GLOBALS.signer.nip44_decrypt(&pubkey, &ciphertext, padded)?;
    let plaintext = String::from_utf8_lossy(&plaintext_bytes);
    println!("{}", plaintext);

    Ok(())
}

pub fn print_event(mut args: env::Args) -> Result<(), Error> {
    let idstr = match args.next() {
        Some(id) => id,
        None => {
            return Err(ErrorKind::Usage(
                "Missing idhex parameter".to_string(),
                "print_event <idhex>".to_owned(),
            )
            .into())
        }
    };

    let id = Id::try_from_hex_string(&idstr)?;

    match GLOBALS.storage.read_event(id)? {
        Some(event) => println!("{}", serde_json::to_string(&event)?),
        None => return Err(ErrorKind::EventNotFound.into()),
    }

    Ok(())
}

pub fn print_relay(mut args: env::Args) -> Result<(), Error> {
    if let Some(url) = args.next() {
        let rurl = RelayUrl::try_from_str(&url)?;
        if let Some(relay) = GLOBALS.storage.read_relay(&rurl)? {
            println!("{}", serde_json::to_string_pretty(&relay)?);
        } else {
            println!("Relay not found.");
        }
        Ok(())
    } else {
        Err(ErrorKind::Usage(
            "Missing url parameter".to_string(),
            "print_relay <url>".to_owned(),
        )
        .into())
    }
}

pub fn print_relays() -> Result<(), Error> {
    let relays = GLOBALS.storage.filter_relays(|_| true)?;
    for relay in &relays {
        println!("{}", serde_json::to_string(relay)?);
    }
    Ok(())
}

pub fn print_followed() -> Result<(), Error> {
    let pubkeys = GLOBALS.storage.get_people_in_list(PersonList::Followed)?;
    for pk in &pubkeys {
        println!("{}", pk.as_hex_string());
    }
    Ok(())
}

pub fn print_muted() -> Result<(), Error> {
    let pubkeys = GLOBALS.storage.get_people_in_list(PersonList::Muted)?;
    for pk in &pubkeys {
        println!("{}", pk.as_hex_string());
    }
    Ok(())
}

pub fn print_person_relays(mut args: env::Args) -> Result<(), Error> {
    let pubkey = match args.next() {
        Some(hex) => PublicKey::try_from_hex_string(&hex, true)?,
        None => {
            return Err(ErrorKind::Usage(
                "Missing pubkeyhex parameter".to_string(),
                "print_person_relays <pubkeyhex>".to_owned(),
            )
            .into())
        }
    };

    let person_relays = GLOBALS.storage.get_person_relays(pubkey)?;
    for record in &person_relays {
        println!("{}", serde_json::to_string(record)?);
    }
    Ok(())
}

pub fn events_of_kind(mut args: env::Args) -> Result<(), Error> {
    let kind: EventKind = match args.next() {
        Some(integer) => integer.parse::<u32>()?.into(),
        None => {
            return Err(ErrorKind::Usage(
                "Missing kind parameter".to_string(),
                "events_of_kind <kind>".to_owned(),
            )
            .into())
        }
    };

    let ids = GLOBALS.storage.find_event_ids(&[kind], &[], None)?;

    for id in ids {
        println!("{}", id.as_hex_string());
    }

    Ok(())
}

pub fn events_of_pubkey_and_kind(mut args: env::Args) -> Result<(), Error> {
    let pubkey = match args.next() {
        Some(hex) => PublicKey::try_from_hex_string(&hex, true)?,
        None => {
            return Err(ErrorKind::Usage(
                "Missing pubkeyhex parameter".to_string(),
                "events_of_pubkey_and_kind <pubkeyhex> <kind>".to_owned(),
            )
            .into())
        }
    };

    let kind: EventKind = match args.next() {
        Some(integer) => integer.parse::<u32>()?.into(),
        None => {
            return Err(ErrorKind::Usage(
                "Missing kind parameter".to_string(),
                "events_of_pubkey_and_kind <pubkeyhex> <kind>".to_owned(),
            )
            .into())
        }
    };

    let ids = GLOBALS.storage.find_event_ids(&[kind], &[pubkey], None)?;

    for id in ids {
        println!("{}", id.as_hex_string());
    }

    Ok(())
}

pub fn ungiftwrap(mut args: env::Args) -> Result<(), Error> {
    let idstr = match args.next() {
        Some(id) => id,
        None => {
            return Err(ErrorKind::Usage(
                "Missing idhex parameter".to_string(),
                "ungiftwrap <idhex>".to_owned(),
            )
            .into())
        }
    };

    let id = Id::try_from_hex_string(&idstr)?;

    let event = match GLOBALS.storage.read_event(id)? {
        Some(event) => {
            if event.kind != EventKind::GiftWrap {
                return Err(ErrorKind::WrongEventKind.into());
            } else {
                event
            }
        }
        None => return Err(ErrorKind::EventNotFound.into()),
    };

    login()?;

    let rumor = GLOBALS.signer.unwrap_giftwrap(&event)?;

    println!("{}", serde_json::to_string(&rumor)?);

    Ok(())
}

pub fn giftwrap_ids() -> Result<(), Error> {
    let ids = GLOBALS
        .storage
        .find_event_ids(&[EventKind::GiftWrap], &[], None)?;

    for id in ids {
        println!("{}", id.as_hex_string());
    }

    Ok(())
}

pub fn reprocess_recent(runtime: &Runtime) -> Result<(), Error> {
    login()?;

    let job = tokio::task::spawn(async move {
        let all_kinds: Vec<EventKind> = EventKind::iter().collect();

        let mut ago = Unixtime::now().unwrap();
        ago.0 -= 86400;

        let events = match GLOBALS
            .storage
            .find_events(&all_kinds, &[], Some(ago), |_| true, false)
        {
            Ok(e) => e,
            Err(e) => {
                println!("ERROR: {}", e);
                vec![]
            }
        };

        let mut count = 0;
        for event in events.iter() {
            if let Err(e) = crate::process::process_new_event(event, None, None, false, true).await
            {
                println!("ERROR: {}", e);
            }
            count += 1;
            if count % 100 == 0 {
                println!("{}...", count);
            }
        }

        println!("Done.");
    });

    Ok(runtime.block_on(job)?)
}

pub fn verify(mut args: env::Args) -> Result<(), Error> {
    let idstr = match args.next() {
        Some(id) => id,
        None => {
            return Err(ErrorKind::Usage(
                "Missing idhex parameter".to_string(),
                "verify <idhex>".to_owned(),
            )
            .into())
        }
    };

    let id = Id::try_from_hex_string(&idstr)?;

    match GLOBALS.storage.read_event(id)? {
        Some(event) => {
            event.verify(None)?;
            println!("Valid event");
        }
        None => return Err(ErrorKind::EventNotFound.into()),
    }

    Ok(())
}

pub fn verify_json(mut args: env::Args) -> Result<(), Error> {
    let json = match args.next() {
        Some(json) => json,
        None => {
            return Err(ErrorKind::Usage(
                "Missing json parameter".to_string(),
                "verify_json <event_json>".to_owned(),
            )
            .into())
        }
    };

    let event: Event = serde_json::from_str(&json)?;
    event.verify(None)?;
    println!("Valid event");

    Ok(())
}

pub fn rebuild_indices() -> Result<(), Error> {
    login()?;

    GLOBALS.storage.rebuild_event_indices()?;
    Ok(())
}

pub fn login() -> Result<(), Error> {
    let mut password = rpassword::prompt_password("Password: ").unwrap();
    let epk = match GLOBALS.storage.read_encrypted_private_key()? {
        Some(epk) => epk,
        None => return Err(ErrorKind::NoPrivateKey.into()),
    };
    GLOBALS.signer.set_encrypted_private_key(epk);
    GLOBALS.signer.unlock_encrypted_private_key(&password)?;
    password.zeroize();
    Ok(())
}
