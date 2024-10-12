#![allow(unused)]

use super::*;
use alloc::boxed::*;
use alloc::collections::VecDeque;
use alloc::vec::*;

pub static MAIL_BOXES: Mutex<Vec<MailBox>> = Mutex::new(Vec::new());

#[derive(Debug)]
pub struct MailBox {
    pub owner: u32, // pid
    pub name: String,
    pub queue: VecDeque<Message>,
}

#[derive(Debug)]
pub struct Message {
    pub from: u32, // pid
    pub data: Box<[u8]>,
}

pub fn create_mail_box(owner: u32, name: String) -> Result<(), ()> {
    for mb in MAIL_BOXES.lock().iter_mut() {
        if mb.name == *name {
            return Err(());
        }
    }

    MAIL_BOXES.lock().push(MailBox {
        owner,
        name,
        queue: VecDeque::new(),
    });
    return Ok(());
}

pub fn delete_mail_box(caller: u32, name: String) {
    let mut mail_boxes = MAIL_BOXES.lock();
    for (i, mb) in mail_boxes.iter_mut().enumerate() {
        if mb.name == *name && mb.owner == caller {
            mail_boxes.remove(i);
            return;
        }
    }
}

pub fn send(from: u32, mail_box: &String, data: &[u8]) -> Result<(), ()> {
    for mb in MAIL_BOXES.lock().iter_mut() {
        if mb.name == *mail_box {
            let mut kdata: Vec<u8> = Vec::with_capacity(data.len());
            kdata.resize(data.len(), 0);
            let mut kdata = kdata.into_boxed_slice();
            kdata.copy_from_slice(data);

            mb.queue.push_back(Message { from, data: kdata });
            return Ok(());
        }
    }
    return Err(());
}

pub fn try_receive(mail_box: &String) -> Result<Option<Message>, ()> {
    for mb in MAIL_BOXES.lock().iter_mut() {
        if mb.name == *mail_box {
            return Ok(mb.queue.pop_front());
        }
    }
    return Err(());
}

/*pub fn receive(mail_box: &String) -> Result<Message, ()> {
    loop {
        let t = try_receive(&mail_box);
        if let Ok(msg) = t {
            if let Some(msg) = msg {
                return Ok(msg);
            }
        } else {
            return Err(());
        }
    }
}*/
