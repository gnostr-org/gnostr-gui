use super::{widgets, GossipUi, Page};
use eframe::{egui, Frame};
use egui::widgets::Button;
use egui::{Context, Label, RichText, Sense, Ui};
use gossip_lib::comms::ToOverlordMessage;
use gossip_lib::{FeedKind, PersonTable, Relay, Table, GLOBALS};

pub(super) fn update(
    app: &mut GossipUi,
    ctx: &Context,
    _frame: &mut Frame,
    ui: &mut Ui,
    local: bool,
) {
    ui.add_space(10.0);
    if local {
        ui.heading("Search notes and users in local database");
    } else {
        ui.heading("Search notes and users on search relays");
    }

    // Warn if there are no search relays configured
    if !local {
        let search_relays: Vec<Relay> = GLOBALS
            .db()
            .filter_relays(|relay| relay.has_usage_bits(Relay::SEARCH))
            .unwrap_or_default();

        if search_relays.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.label("You must first configure SEARCH relays on the ");
                if ui.link("relays").clicked() {
                    app.set_page(ctx, Page::RelaysKnownNetwork(None));
                }
                ui.label(" page.");
            });
            return;
        }
    }

    ui.add_space(12.0);

    let mut trigger_search = false;

    ui.horizontal(|ui| {
        let response = ui.add(
            text_edit_line!(app, app.search)
                .hint_text("Search for People and Notes")
                .desired_width(600.0),
        );

        if app.entering_a_search_page {
            // Focus on the search input
            response.request_focus();

            app.entering_a_search_page = false;
        }

        if ui.add(Button::new("Search")).clicked() {
            trigger_search = true;
        }
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            trigger_search = true;
        }
    });

    if trigger_search {
        if local {
            let _ = GLOBALS
                .to_overlord
                .send(ToOverlordMessage::SearchLocally(app.search.clone()));
        } else {
            let _ = GLOBALS
                .to_overlord
                .send(ToOverlordMessage::SearchRelays(app.search.clone()));
        }
    }

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);

    let people = GLOBALS.people_search_results.read().clone();
    let notes = GLOBALS.note_search_results.read().clone();

    app.vert_scroll_area().show(ui, |ui| {
        if !people.is_empty() {
            for person in people.iter() {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    // Avatar first
                    let avatar = if let Some(avatar) = app.try_get_avatar(ctx, &person.pubkey) {
                        avatar
                    } else {
                        app.placeholder_avatar.clone()
                    };
                    if widgets::paint_avatar(ui, person, &avatar, widgets::AvatarSize::Feed)
                        .clicked()
                    {
                        app.set_page(ctx, Page::Person(person.pubkey));
                    };

                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(gossip_lib::names::pubkey_short(&person.pubkey)).weak(),
                        );
                        GossipUi::render_person_name_line(app, ui, person, false);
                    });
                });
            }
        }

        if !notes.is_empty() {
            for event in notes.iter() {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(crate::date_ago::date_ago(event.created_at))
                            .italics()
                            .weak(),
                    );

                    if let Ok(Some(person)) = PersonTable::read_record(event.pubkey, None) {
                        GossipUi::render_person_name_line(app, ui, &person, false);
                    } else {
                        ui.label(event.pubkey.as_bech32_string());
                    }
                });

                let mut summary = event
                    .content
                    .get(0..event.content.len().min(100))
                    .unwrap_or("...")
                    .replace('\n', " ");

                if summary.is_empty() {
                    // Show something they can click on anyways
                    summary = "[no event summary]".to_owned();
                }

                if ui.add(Label::new(summary).sense(Sense::click())).clicked() {
                    app.set_page(
                        ctx,
                        Page::Feed(FeedKind::Thread {
                            id: event.id,
                            referenced_by: event.id,
                            author: Some(event.pubkey),
                        }),
                    );
                }
            }
        }

        if people.is_empty() && notes.is_empty() {
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label("No results found.");
        }
    });
}
