use crate::context::Context;
use common::messages::Response;
use dpi::frame::FrameType;

pub fn data(ctx: &mut Context, response: Response) {
    let frame = match response {
        Response::Data(frame) => frame,
        _ => {
            log::error!("Response: Wrong flow for this type of data chosen");
            return;
        },
    };

    match frame {
        FrameType::Metadata(metadata) => {
            // TODO: Process parsed metadata
        },
        FrameType::Header(header) => {
            // TODO: ...
        }
        FrameType::Raw(frame) => {
            if !ctx.client_settings.unparsed_frames_drop {
                ctx.net_storage.raw.add(frame);
                // TODO: Handle raw frames
            }
            // Else - pass
        },
    }
}

pub fn process(ctx: &mut Context, response: Response) {
    match response {
        Response::ServerSettings(dto) => process::server_settings(ctx, dto),
        Response::SuccessChangePassword => {
            modals::success::password_changed(&ctx.modals_tx)
        },
        Response::SuccessSaveConfig => modals::success::config_saved(&ctx.modals_tx),
        Response::SuccessSetCompression(is_enabled) => {
            modals::success::compression_set(&ctx.modals_tx, is_enabled)
        },
        Response::SuccessSetInterface(new) => {
            modals::success::interface_set(&ctx.modals_tx, new)
        },
        Response::SuccessSetSendUnparsedFrames(is_enabled) => {
            modals::success::send_unparsed_frames_set(&ctx.modals_tx, is_enabled)
        },
        Response::SuccessSync => process::pong(ctx),
        Response::Error(error) => modals::error::try_send(&ctx.modals_tx, error),

        Response::Data(_) => {
            log::error!("Response: Wrong flow for this type of data chosen");
        },
    }
}

mod modals {
    use crate::ui::modals::Modal;

    type Sender = crossbeam::channel::Sender<Box<dyn Modal>>;

    pub mod error {
        use crate::communication::response::modals::Sender;
        use crate::ui::modals::message::MessageModal;
        use common::messages::ServerError;

        pub fn try_send(tx: &Sender, error: ServerError) {
            MessageModal::error(&localize(&error)).try_send_by(tx);
        }

        fn localize(err: &ServerError) -> String {
            match err {
                ServerError::FailedToChangePassword => {
                    t!("Response.Error.PasswordChange").to_string()
                },
                ServerError::FailedToGetInterfaces => {
                    t!("Response.Error.InterfacesGet").to_string()
                },
                ServerError::FailedToSaveConfig => {
                    t!("Response.Error.ConfigSave").to_string()
                },
                ServerError::InvalidMessageFormat => {
                    t!("Response.Error.InvalidMessageFormat").to_string()
                },
                ServerError::InvalidInterface => {
                    t!("Response.Error.InvalidInterface").to_string()
                },
                ServerError::MutexPoisoned => {
                    t!("Response.Error.MutexPoisoned").to_string()
                },
            }
        }
    }

    pub mod success {
        use crate::communication::response::modals::Sender;
        use crate::ui::modals::message::MessageModal;

        pub fn compression_set(tx: &Sender, is_enabled: bool) {
            let text: String = if is_enabled {
                t!("Response.SetCompression.Success.On").to_string()
            } else {
                t!("Response.SetCompression.Success.Off").to_string()
            };
            MessageModal::info(&text).try_send_by(tx);
        }

        pub fn config_saved(tx: &Sender) {
            MessageModal::info(&t!("Response.SaveConfig.Success")).try_send_by(tx);
        }

        pub fn interface_set(tx: &Sender, new: String) {
            MessageModal::info(&t!("Response.SetInterface.Success", "interface" = new))
                .try_send_by(tx);
        }

        pub fn password_changed(tx: &Sender) {
            MessageModal::info(&t!("Response.PasswordChange.Success")).try_send_by(tx);
        }

        pub fn send_unparsed_frames_set(tx: &Sender, is_enabled: bool) {
            let text: String = if is_enabled {
                t!("Response.SetSendUnparsedFrames.Success.On").to_string()
            } else {
                t!("Response.SetSendUnparsedFrames.Success.Off").to_string()
            };
            MessageModal::info(&text).try_send_by(tx);
        }
    }
}

mod process {
    use crate::context::{Context, ServerSettings};
    use chrono::Local;
    use common::messages::ServerSettingsDto;

    pub fn pong(ctx: &mut Context) {
        ctx.heartbeat.update();
    }

    pub fn server_settings(ctx: &mut Context, dto: ServerSettingsDto) {
        ctx.settings_server = ServerSettings {
            compression_active: dto.compression_active,
            compression_config: dto.compression_config,

            interfaces_available: dto.interfaces_available,
            interface_active: dto.interface_active,
            interface_config: dto.interface_config,

            link_type: dto.link_type.map(pcap::Linktype),

            send_unparsed_frames_active: dto.send_unparsed_frames_active,
            send_unparsed_frames_config: dto.send_unparsed_frames_config,

            last_updated: Some(Local::now()),
        };
    }
}
