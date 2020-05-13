#![cfg_attr(not(feature = "gtk_3_10"), allow(unused_variables, unused_mut))]

extern crate gio;
extern crate glib;
extern crate gtk;

use serde::{Serialize, Deserialize};

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::str::from_utf8;

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
use crate::model::settings::Settings;
//, tx : Sender<String>
fn create_message_widget(message: Message) -> gtk::TextView
{
    let textview = gtk::TextView::new();
    // view settings
    textview.set_wrap_mode(gtk::WrapMode::Word);
    textview.set_justification(gtk::Justification::Left);
    textview.set_indent(2);
    textview.set_editable(false);
    // Format output here
    let textbuffer = textview.get_buffer().unwrap();
    textbuffer.set_text(format!("{0}\n{1}\n{2}\n", message.datetime, message.name, message.body).as_str());
    return textview;
}
fn create_message_widget_from_ref(message: Rc<RefCell<Message>>) -> gtk::TextView
{
    let textview = gtk::TextView::new();
    // view settings
    textview.set_wrap_mode(gtk::WrapMode::Word);
    textview.set_justification(gtk::Justification::Right);
    textview.set_indent(2);
    textview.set_editable(false);

    let textbuffer = textview.get_buffer().unwrap();
    textbuffer.set_text(format!("{0}\n{1}\n{2}\n",
                                message.as_ref().borrow().borrow().datetime,
                                message.as_ref().borrow().borrow().name,
                                message.as_ref().borrow().borrow().body).as_str());
    return textview;
}

pub fn handle_stream(mut stream: TcpStream) -> Message
{
    let mut raw_bytes = vec![];
    stream.read_to_end(&mut raw_bytes).unwrap();
    let data = String::from_utf8( raw_bytes ).unwrap();
    let mut msg: Message = serde_json::from_str(&data).unwrap();
    msg.name += format!(" {}", stream.peer_addr().unwrap()).as_str().clone();
    drop(stream);
    return msg;
}

pub fn build_ui(application: &gtk::Application)
{
    let glade_src = include_str!("chat.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: gtk::Window = builder.get_object("window1").expect("Couldn't get window");
    window.set_application(Some(application));

    let mut message = Rc::new(RefCell::new(Message::new()));

    let mut settings = Rc::new(RefCell::new(Settings
        {
            name: String::from("John"),
            port: String::from("27015"),
            connection_string: String::from("TBD"),
            db_ip: String::from("TBD")
        }));

    let mut current_connection = Rc::new(RefCell::new(Connection
        {
            ip: String::from("127.0.0.1"),
            port: String::from("27015")
        }));

    // init settings dialog
    let toolbar: gtk::Toolbar = builder.get_object("toolbar1").expect("Couldn't get toolbar1");
    let settings_button: gtk::ToolButton = builder.get_object("settings_button1").expect("Couldn't get settings_button");

    settings_button.connect_clicked(clone!(@strong settings, @weak window, @strong builder => move |_|
        {
            let set_dialog: gtk::Dialog = builder.get_object("settings_dialog").expect("Couldn't get settings dialog");
            set_dialog.run();
            set_dialog.hide();
            window.show_all();
        }));

    let set_name_entry: gtk::Entry = builder.get_object("set_name_entry").expect("Couldn't get set_name_entry");
    let set_port_entry: gtk::Entry = builder.get_object("set_port_entry").expect("Couldn't get set_port_entry");
    let set_database_ip: gtk::Entry = builder.get_object("set_database_ip").expect("Couldn't get set_database_ip");
    let set_conn_str: gtk::Entry = builder.get_object("set_conn_str").expect("Couldn't get set_conn_str");

    set_name_entry.connect_changed(clone!(@strong settings, @weak set_name_entry, @weak message => move |_|
        {
            settings.borrow_mut().name = set_name_entry.get_text().expect("Couldn't get text from set_name_entry").to_string();
            message.borrow_mut().name = set_name_entry.get_text().expect("Couldn't get text from set_name_entry").to_string();
        }));
    set_port_entry.connect_changed(clone!(@strong settings, @weak set_port_entry => move |_|
        {
            settings.borrow_mut().port = set_port_entry.get_text().expect("Couldn't get text from set_port_entry").to_string();
        }));
    set_database_ip.connect_changed(clone!(@strong settings, @weak set_database_ip => move |_|
        {
            settings.borrow_mut().db_ip = set_database_ip.get_text().expect("Couldn't get text from set_database_ip").to_string();
        }));
    set_conn_str.connect_changed(clone!(@strong settings, @weak set_conn_str => move |_|
        {
            settings.borrow_mut().name = set_conn_str.get_text().expect("Couldn't get text from set_conn_str").to_string();
        }));

    let set_dialog: gtk::Dialog = builder.get_object("settings_dialog").expect("Couldn't get settings dialog");
    set_dialog.run();
    set_dialog.hide();
    // TODO: init settings
    println!("{}", settings.as_ref().borrow().borrow().port.clone());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", settings.as_ref().borrow().borrow().port)).unwrap();
    message.borrow_mut().name = settings.as_ref().borrow().borrow().name.clone();
    let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    // this item used to storage message fields in order to send it
    // TODO: set message field name to name from settings

    let message_entry: Entry = builder.get_object("message_entry").expect("Couldn't get message entry field");
    message_entry.connect_changed(clone!(@strong message, @weak message_entry => move |_|
        {
            message.borrow_mut().body = message_entry.get_text().expect("Couldn't get text from message entry").to_string();
        }));

    let open_chat: gtk::Button = builder.get_object("open_chat").expect("Couldn't get open_chat button");
    let create_connection: gtk::Button = builder.get_object("create_connection").expect("Couldn't get create_connection button");
    let ip_entry: gtk::Entry = builder.get_object("ip_entry").expect("Couldn't get ip_entry");
    let port_entry: gtk::Entry = builder.get_object("port_entry").expect("Couldn't get port_entry");

    create_connection.connect_clicked(clone!(@strong current_connection, @weak window, @weak ip_entry, @weak port_entry, @weak builder => move |_|
        {
            let ip = Rc::new(ip_entry.get_text().expect("Couldn't get text from ip entry").to_string());
            let port = Rc::new(port_entry.get_text().expect("Couldn't get text from port entry").to_string());
            ip_entry.set_text("");
            port_entry.set_text("");
            // TODO: create connection and store in button action under new chat button

            let connection_box: gtk::Box = builder.get_object("box1").expect("Couldn't get box1");
            let button = gtk::Button::new();
            button.set_label(format!("{}:{}", ip.clone(), port.clone()).as_str());
            button.connect_clicked(clone!(@strong ip, @strong port, @strong current_connection => move |_| {
                current_connection.borrow_mut().ip = (*ip).clone();
                current_connection.borrow_mut().port = (*port).clone();
            }));
            connection_box.pack_start(&button, false, false, 0u32);

            current_connection.borrow_mut().ip = (*ip).clone();
            current_connection.borrow_mut().port = (*port).clone();
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

    send_button.connect_clicked(clone!(@strong message, @strong current_connection, @weak message_entry, @weak message_box, @weak window => move |_|
        {
            let ip = current_connection.as_ref().borrow().borrow().ip.clone();
            let port = current_connection.as_ref().borrow().borrow().port.clone();
            let message_clone = message.as_ref().borrow().borrow().clone();
            message.borrow_mut().update_time();
            // TODO: send message
            match TcpStream::connect(format!("{0}:{1}", ip, port)) {
                Ok(mut stream) => {
                    println!("Successfully connected to server");

                    let msg = serde_json::to_string(message.as_ref()).unwrap();

                    stream.write(msg.as_str().as_bytes()).unwrap();
                    println!("{}", msg);
                    println!("Sent message");
                },
                Err(e) => {
                    println!("Failed to connect: {}", e);
                }
            }

            let textview = create_message_widget_from_ref(message.clone());
            // TODO: write to db here

            message_box.pack_start(&textview, false, false, 0u32);
            message_entry.set_text("");
            window.show_all();
        }));
    // TODO: New chat dialog box

    window.show_all();

    enum MessageReceived {
        CreateMessage(Message),
    }

    thread::spawn(move||{
        let val = String::from("Listener thread started");
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap().clone());
                    let mut msg = handle_stream(stream);
                    let _ = sender.send(MessageReceived::CreateMessage(msg));
                }
                Err(e) => {
                    println!("Error: {}", e);
                    /* connection failed */
                }
            }
        }
        drop(listener);
    }
    );

    let message_box_clone = message_box.clone();
    receiver.attach(None, move |msg| {
        match msg {
            MessageReceived::CreateMessage(msg) => {
                let textview = create_message_widget(msg);
                // Adding new message inside message_box
                message_box_clone.pack_start(&textview, false, false, 0u32);
                // Update form with new widget
                window.show_all();
            },
        }

        // Returning false here would close the receiver
        // and have senders fail
        glib::Continue(true)
    });
}

