use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let submit_on_enter = ctx.link().batch_callback(|e: KeyboardEvent| {
            if e.key() == "Enter" {
                Some(Msg::SubmitMessage)
            } else {
                None
            }
        });

        html! {
            <div class="flex w-screen h-screen bg-gray-50">
                <div class="flex-none w-64 h-screen bg-white shadow-md">
                    <div class="p-4 border-b border-gray-200">
                        <div class="text-xl font-bold text-gray-800 flex items-center">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 mr-2 text-blue-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 8h2a2 2 0 012 2v6a2 2 0 01-2 2h-2v4l-4-4H9a2 2 0 01-2-2v-6a2 2 0 012-2h10" />
                            </svg>
                            {"Chat App"}
                        </div>
                    </div>
                    <div class="overflow-y-auto">
                        <div class="p-3 text-xs font-bold text-gray-500 uppercase">{"Active Users"}</div>
                        {
                            if self.users.is_empty() {
                                html! {
                                    <div class="flex justify-center p-4 text-gray-500">
                                        <div class="flex flex-col items-center">
                                            <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
                                            </svg>
                                            <span class="mt-2">{"No users online"}</span>
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {
                                    <>
                                    {
                                        self.users.clone().iter().map(|u| {
                                            html!{
                                                <div class="flex items-center p-3 hover:bg-gray-50 cursor-pointer">
                                                    <div class="relative">
                                                        <img class="w-12 h-12 rounded-full shadow-sm" src={u.avatar.clone()} alt="avatar"/>
                                                        <div class="absolute bottom-0 right-0 w-3 h-3 bg-green-500 rounded-full border-2 border-white"></div>
                                                    </div>
                                                    <div class="ml-3">
                                                        <div class="text-sm font-medium">{u.name.clone()}</div>
                                                        <div class="text-xs text-gray-500">{"Online"}</div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Html>()
                                    }
                                    </>
                                }
                            }
                        }
                    </div>
                </div>
                <div class="grow h-screen flex flex-col bg-white border-l border-gray-200">
                    <div class="w-full h-16 border-b border-gray-200 flex items-center px-4 shadow-sm">
                        <div class="flex items-center">
                            <div class="flex items-center justify-center w-10 h-10 bg-blue-500 rounded-full text-white">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 8h2a2 2 0 012 2v6a2 2 0 01-2 2h-2v4l-4-4H9a2 2 0 01-2-2v-6a2 2 0 012-2h10" />
                                </svg>
                            </div>
                            <div class="ml-3">
                                <div class="text-lg font-medium">{"Chat Room"}</div>
                                <div class="text-xs text-gray-500">{format!("{} participants", self.users.len())}</div>
                            </div>
                        </div>
                    </div>
                    <div class="w-full grow overflow-auto p-4 bg-gray-50">
                        {
                            if self.messages.is_empty() {
                                html! {
                                    <div class="flex flex-col items-center justify-center h-full text-gray-500">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16 mb-4 text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                                        </svg>
                                        <p>{"Start a conversation!"}</p>
                                    </div>
                                }
                            } else {
                                html! {
                                    <>
                                    {
                                        self.messages.iter().map(|m| {
                                            let is_current_user = m.from == "You";
                                            
                                            let avatar_url = if let Some(user) = self.users.iter().find(|u| u.name == m.from) {
                                                user.avatar.clone()
                                            } else {
                                                format!("https://avatars.dicebear.com/api/adventurer-neutral/{}.svg", m.from)
                                            };
                                            
                                            html!{
                                                <div class={format!("flex mb-4 {}", if is_current_user { "justify-end" } else { "" })}>
                                                    {
                                                        if !is_current_user {
                                                            html! {
                                                                <div class="flex-shrink-0 mr-3">
                                                                    <img class="w-8 h-8 rounded-full shadow-sm" src={avatar_url} alt="avatar"/>
                                                                </div>
                                                            }
                                                        } else {
                                                            html! {}
                                                        }
                                                    }
                                                    <div class={format!("max-w-xs md:max-w-md p-3 rounded-lg shadow-sm {}", 
                                                        if is_current_user { 
                                                            "bg-blue-500 text-white rounded-br-none" 
                                                        } else { 
                                                            "bg-white rounded-bl-none" 
                                                        })}>
                                                        {
                                                            if !is_current_user {
                                                                html! {
                                                                    <div class="font-medium text-sm mb-1">{m.from.clone()}</div>
                                                                }
                                                            } else {
                                                                html! {}
                                                            }
                                                        }
                                                        <div class={format!("text-sm {}", if is_current_user { "text-white" } else { "text-gray-800" })}>
                                                            if m.message.ends_with(".gif") || m.message.contains("giphy.com") {
                                                                <img class="mt-2 rounded-lg max-w-full" src={m.message.clone()} alt="gif"/>
                                                            } else {
                                                                {m.message.clone()}
                                                            }
                                                        </div>
                                                        <div class={format!("text-xs mt-1 text-right {}", if is_current_user { "text-blue-100" } else { "text-gray-400" })}>
                                                            {"now"}
                                                        </div>
                                                    </div>
                                                    {
                                                        if is_current_user {
                                                            html! {
                                                                <div class="flex-shrink-0 ml-3">
                                                                    <img class="w-8 h-8 rounded-full shadow-sm" src="https://avatars.dicebear.com/api/adventurer-neutral/you.svg" alt="avatar"/>
                                                                </div>
                                                            }
                                                        } else {
                                                            html! {}
                                                        }
                                                    }
                                                </div>
                                            }
                                        }).collect::<Html>()
                                    }
                                    </>
                                }
                            }
                        }
                    </div>
                    <div class="w-full p-4 bg-white border-t border-gray-200">
                        <div class="flex">
                            <input 
                                ref={self.chat_input.clone()} 
                                type="text" 
                                placeholder="Type a message..." 
                                class="flex-1 py-2 px-4 bg-gray-100 rounded-l-full outline-none focus:ring-2 focus:ring-blue-400 focus:bg-white"
                                onkeyup={submit_on_enter}
                            />
                            <button 
                                onclick={submit} 
                                class="px-6 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-r-full transition-colors focus:outline-none"
                            >
                                <div class="flex items-center">
                                    <span class="mr-2 hidden sm:inline">{"Send"}</span>
                                    <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 fill-white">
                                        <path d="M0 0h24v24H0z" fill="none"></path><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                                    </svg>
                                </div>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}