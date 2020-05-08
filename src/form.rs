#![cfg_attr(not(feature = "gtk_3_10"), allow(unused_variables, unused_mut))]

extern crate gio;
extern crate glib;
extern crate gtk;

use glib::clone;
use gtk::prelude::*;
use gtk::prelude::EntryExt;
use gtk::{Builder, Entry};
use gtk::prelude::GridExt;

use std::cell::{RefCell, Cell};
use std::rc::Rc;

use std::borrow::Borrow;
use crate::model::message::Message;
use crate::model::connection::Connection;

pub fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("chat.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: gtk::Window = builder.get_object("window1").expect("Couldn't get window");
    window.set_application(Some(application));

    // TODO: init settings



    //this item used to storage message fields in order to send it
    // TODO: set message field name to name from settings
    let mut message = Rc::new(RefCell::new(Message::new()));
    let message_entry: Entry = builder.get_object("message_entry").expect("Couldn't get message entry field");
    message_entry.connect_changed(clone!(@strong message, @weak message_entry => move |_|
        {
            message.borrow_mut().body = message_entry.get_text().expect("Couldn't get text from message entry").to_string();
            message_entry.set_text("");
        }));

    let mut current_connection = Rc::new(RefCell::new(Connection::new())); // Would change by swapping tabs with connection
    let open_chat: gtk::Button = builder.get_object("open_chat").expect("Couldn't get open_chat button");
    let create_connection: gtk::Button = builder.get_object("create_connection").expect("Couldn't get create_connection button");
    let ip_entry: gtk::Entry = builder.get_object("ip_entry").expect("Couldn't get ip_entry");
    let port_entry: gtk::Entry = builder.get_object("port_entry").expect("Couldn't get port_entry");

    create_connection.connect_clicked(clone!(@strong current_connection, @weak window, @weak ip_entry, @weak port_entry => move |_|
        {
            let ip = ip_entry.get_text().expect("Couldn't get text from ip entry").to_string();
            let port = port_entry.get_text().expect("Couldn't get text from port entry").to_string();
            ip_entry.set_text("");
            port_entry.set_text("");
            // TODO: create connection and store in button action under new chat button
        }));

    open_chat.connect_clicked(clone!(@strong current_connection, @weak window, @strong builder => move |_|
        {
            let dialog: gtk::Dialog = builder.get_object("dialog1").expect("Couldn't get dialog");
            dialog.run();
            dialog.hide();
            window.show_all();
        }));

    let send_button: gtk::Button = builder.get_object("send_button").expect("Couldn't get send button");
    let message_box: gtk::Box = builder.get_object("box3").expect("Couldn't get message box");

    send_button.connect_clicked(clone!(@strong message, @strong current_connection, @weak window => move |_|
        {
            let ip = current_connection.as_ref().borrow().borrow().ip.clone();
            // TODO: send message
            let textview = gtk::TextView::new();
            // view settings
            textview.set_wrap_mode(gtk::WrapMode::Word);
            textview.set_justification(gtk::Justification::Right);
            textview.set_indent(2);

            let textbuffer = textview.get_buffer().unwrap();
            textbuffer.set_text(format!("{0}\n{1}\n{2}\n", message.as_ref().borrow().borrow().datetime,
            message.as_ref().borrow().borrow().name, message.as_ref().borrow().borrow().body).as_str());
            // TODO: write to db here

            message_box.pack_start(&textview, false, false, 0u32);
            window.show_all();
        }));
    // TODO: New chat dialog box
    window.show_all();
}

