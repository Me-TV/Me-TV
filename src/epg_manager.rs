/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2019, 2020  Russel Winder
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use std::panic;

use glib;
use glib::translate::{from_glib, ToGlib};

use gst;
use gst_mpegts;

use crate::control_window::Message;
use crate::channels_data::add_logical_channel_number_for_service_id;

static PRINT_BAT: bool = false;
static PRINT_CAT: bool = false;
static PRINT_EIT: bool = false;
static PRINT_NIT: bool = false;
static PRINT_PAT: bool = false;
static PRINT_PMT: bool = false;
static PRINT_SDT: bool = false;
static PRINT_TDT: bool = false;
static PRINT_TSDT: bool = false;
static PRINT_TOT: bool = false;

static PRINT_DATA: bool = false;

fn build_bat(bat: &gst_mpegts::BAT, to_cw: &glib::Sender<Message>) {
    // Do not seem to get any of these on BBC News on Freeview from Crystal Palace.
    if PRINT_BAT {
        println!("========  BAT section.");
    }
    for descriptor in bat.get_descriptors().iter() {
        if PRINT_BAT {
            println!("         {:?}", descriptor);
        }
    }
    for stream in bat.get_streams().iter() {
        if PRINT_BAT {
            println!("         {:?}", stream);
        }
    }
}

fn build_cat(cat: &Vec<gst_mpegts::Descriptor>, to_cw: &glib::Sender<Message>) {
    // Do not seem to get any of these on BBC News on Freeview from Crystal Palace.
    if PRINT_CAT {
        println!("========  CAT section:  {:?}", &cat);
    }
}

fn build_eit(eit: &gst_mpegts::EIT, to_cw: &glib::Sender<Message>) {
    if PRINT_EIT {
        println!("========  EIT section.");
    }
    for event in eit.get_events().iter() {
        if PRINT_EIT {
            println!("    EITEvent:  event_id = {:?}", event.get_event_id());
        }
        for d in event.get_descriptors().iter() {
            match d.get_tag() {
                gst_mpegts::DVBDescriptorType::Component => {
                    let component = d.parse_dvb_component().unwrap();
                    if PRINT_EIT {
                        println!("            Component  {:?}", &component);
                    }
                },
                gst_mpegts::DVBDescriptorType::Content => {
                    let c = d.parse_dvb_content().unwrap();
                    for item in c.iter() {
                        if PRINT_EIT {
                            println!("            {}", gst_mpegts::content_description(item.get_content_nibble_1().to_glib(), item.get_content_nibble_2()))
                        }
                    }
                },
                gst_mpegts::DVBDescriptorType::ContentIdentifier => {
                    if PRINT_EIT {
                        println!("            Unknown processing technique");
                    }
                },
                gst_mpegts::DVBDescriptorType::Linkage => {
                    let linkage = d.parse_dvb_linkage().unwrap();
                    if PRINT_EIT {
                        println!("            Linkage  {:?}", &linkage);
                    }
                }
                gst_mpegts::DVBDescriptorType::ShortEvent => {
                    // TODO It seems this can panic
                    //   assertion failed: !ptr.is_null()', /home/users/russel/.cargo/git/checkouts/glib-928cf7b282977403/3a64675/src/gstring.rs:51:9
                    //   on real data from Freeview.
                    //
                    // See https://gitlab.freedesktop.org/gstreamer/gst-plugins-bad/-/issues/1333
                    match panic::catch_unwind(|| {
                        let (language_code, title, blurb) = d.parse_dvb_short_event().unwrap();
                        if PRINT_EIT {
                            println!("            {}, {}, {}", &language_code, &title, &blurb);
                        }
                    }) {
                        Ok(_) => {},
                        Err(_) => println!("************  parse_dvb_short_event panicked, assume there is a 0x1f encoding byte in the string."),
                    }
                },
                gst_mpegts::DVBDescriptorType::PrivateDataSpecifier => {
                    if PRINT_EIT {
                        println!("            PrivateDataSpecifier  {:?}", &d.parse_dvb_private_data_specifier());
                    }
                },
                gst_mpegts::DVBDescriptorType::FtaContentManagement => {
                    if PRINT_EIT {
                        println!("            FtaContentManagement  Unknown processing technique");
                    }
                },
                x => {
                    match x.to_glib() {
                        0x89 => {
                            // Infer this is actually a MiscDescriptorType in the user defined range.
                            //
                            // Looks to be one or two u8 followed by a string data.
                            // It seems that the first byte may be a boolean to say whether
                            // or not there is a second byte before the string data.
                            // It seems the string data is a three character
                            // language code followed by an encoded message.
                            let data = d.get_data();
                            let mut i = 2;
                            let is_second_byte = data[i] == 1;
                            i += 1;
                            if is_second_byte {
                                let _second_byte = data[i];
                                i += 1;
                            }
                            let language_code = String::from_utf8_lossy(&data[i..(i+3)]).to_string();
                            i += 3;
                            let (encoding, count) = gst_mpegts::select_encoding(data[i..(i+3)].to_vec());
                            i += count;
                            let message = match encoding {
                                Some(e) =>  e.decode(&data[i..]).0.to_string(),
                                None => "String with language code could not be processed.".to_string(),
                            };
                            if PRINT_EIT {
                                println!("            MiscDescriptorType::__Unknown({}):  language_code = {}, private_data = {}", &x.to_glib(), &language_code, &message);
                            }
                            // The messages all seem to be warnings about the programs.
                        },
                        y => println!("************  Got an EIT tag: {:?}", y),
                    }
                },
            }
            if PRINT_DATA {
                println!("        {:?}", d.get_data());
            }
        }
    }
}

fn build_nit(nit: &gst_mpegts::NIT, to_cw: &glib::Sender<Message>) {
    if PRINT_NIT {
        println!("========  NIT section: actual_network = {}, network_id = {}", nit.get_actual_network(), nit.get_network_id());
    }
    for descriptor in nit.get_descriptors().iter() {
        // EN 300 468 Table 12 states which descriptors are allowed.
        match descriptor.get_tag() {
            gst_mpegts::DVBDescriptorType::NetworkName => {
                let name = descriptor.parse_dvb_network_name().unwrap();
                if PRINT_NIT {
                    println!("    NetworkName:  {}", &name);
                }
            },
            gst_mpegts::DVBDescriptorType::Extension => {
                match descriptor.get_tag_extension().unwrap() {
                    gst_mpegts::DVBExtendedDescriptorType::TargetRegionName => {
                        let target_region_name = descriptor.parse_target_region_name().unwrap();
                        if PRINT_NIT {
                            println!("    Extension:  TargetRegionName:   country_code = {}, iso_639_language_code = {}, region_data = {:?}",
                                     &target_region_name.get_country_code(),
                                     &target_region_name.get_iso_639_language_code(),
                                     &target_region_name.get_region_data());
                        }
                    },
                    gst_mpegts::DVBExtendedDescriptorType::TargetRegion => {
                        let target_region = &descriptor.parse_target_region().unwrap();
                        if PRINT_NIT {
                            println!("    Extension:  TargetRegion:  country_code = {}, additional_country_codes = {:?}",
                                     &target_region.get_country_code(),
                                     &target_region.get_additional_country_codes());
                        }
                    },
                    gst_mpegts::DVBExtendedDescriptorType::Message => {
                        let message = descriptor.parse_message().unwrap();
                        if PRINT_NIT {
                            println!("    Extension:  Message:  message_id = {}, iso_639_language_code = {}, message = {}",
                                     &message.get_message_id(),
                                     &message.get_iso_639_language_code(),
                                     &message.get_message());
                        }
                    },
                    gst_mpegts::DVBExtendedDescriptorType::UriLinkage => {
                        let uri_linkage = descriptor.parse_uri_linkage().unwrap();
                        if PRINT_NIT {
                            println!("    Extension:  UriLinkage:  uri_linkage_type = {:?}, uri = {}, min_polling_interval = {}, private_data = {:?}",
                                     &uri_linkage.get_uri_linkage_type(),
                                     &uri_linkage.get_uri(),
                                     &uri_linkage.get_min_polling_interval(),
                                     &uri_linkage.get_private_data());
                        }
                    },
                    x => println!("************  Got an extended descriptor type {:?}", x),
                }
            },
            gst_mpegts::DVBDescriptorType::Linkage => {
                let linkage = descriptor.parse_dvb_linkage().unwrap();
                if PRINT_NIT {
                    println!("    Linkage:  transport_stream_id = {}, original_network_id = {}, service_id = {}, linkage_type = {:?}",
                             linkage.get_transport_stream_id(),
                             linkage.get_original_network_id(),
                             linkage.get_service_id(),
                             linkage.get_linkage_type());
                }
                // TODO get the event, extended event, mobile hand over
                // TODO get private data.
            },
            gst_mpegts::DVBDescriptorType::PrivateDataSpecifier => {
                // It seems that this is the original_network_id being presented at the NIT section level.
                let private_data = descriptor.parse_dvb_private_data_specifier().unwrap();
                if PRINT_NIT {
                    println!("    PrivateDataSpecifier: {}, {:?}", &private_data.0, &private_data.1);
                }
            },
            x => println!("************  Got a descriptor type {:?}", x),
        }
        if PRINT_DATA {
            println!("        {:?}", descriptor.get_data());
        }
    }
    for stream in nit.get_streams().iter() {
        if PRINT_NIT {
            println!("    NITStream:  transport_stream_id = {}, original_network_id = {}", stream.get_transport_stream_id(), stream.get_original_network_id());
        }
        for descriptor in stream.get_descriptors().iter() {
            match descriptor.get_tag() {
                gst_mpegts::DVBDescriptorType::ServiceList => {
                    let service_list = descriptor.parse_dvb_service_list().unwrap();
                    if PRINT_NIT {
                        println!("        ServiceList:");
                    }
                    for service in service_list.iter() {
                        if PRINT_NIT {
                            println!("            service_id = {}, service_type = {:?}",
                                     &service.get_service_id(),
                                     &service.get_type());
                        }
                    }
                },
                gst_mpegts::DVBDescriptorType::TerrestrialDeliverySystem => {
                    let terrestrial_delivery_system = descriptor.parse_terrestrial_delivery_system().unwrap();
                    if PRINT_NIT {
                        println!("    TerrestrialDeliverySystem: \
frequency = {}, bandwidth = {}, priority = {}, time_slicing = {}, mpe_fec = {}, constellation = {:?}, hierarchy = {:?}, \
code_rate_hp = {:?}, code_rate_lp = {:?}, guard_interval = {:?}, transmission_mode = {:?}, other_frequency = {}",
                                 &terrestrial_delivery_system.get_frequency(),
                                 &terrestrial_delivery_system.get_bandwidth(),
                                 &terrestrial_delivery_system.get_priority(),
                                 &terrestrial_delivery_system.get_time_slicing(),
                                 &terrestrial_delivery_system.get_mpe_fec(),
                                 &terrestrial_delivery_system.get_constellation(),
                                 &terrestrial_delivery_system.get_hierarchy(),
                                 &terrestrial_delivery_system.get_code_rate_hp(),
                                 &terrestrial_delivery_system.get_code_rate_lp(),
                                 &terrestrial_delivery_system.get_guard_interval(),
                                 &terrestrial_delivery_system.get_transmission_mode(),
                                 &terrestrial_delivery_system.get_other_frequency());
                    }
                },
                gst_mpegts::DVBDescriptorType::Extension => {
                    match descriptor.get_tag_extension().unwrap() {
                        gst_mpegts::DVBExtendedDescriptorType::TargetRegion => {
                            let target_region = descriptor.parse_target_region().unwrap();
                            if PRINT_NIT {
                                println!("        Extension:  TargetRegion:  country_code = {}, additional_country_codes = {:?}",
                                         &target_region.get_country_code(),
                                         &target_region.get_additional_country_codes());
                            }
                        },
                        gst_mpegts::DVBExtendedDescriptorType::T2DeliverySystem => {
                            let t2_delivery_system = descriptor.parse_dvb_t2_delivery_system().unwrap();
                            if PRINT_NIT {
                                println!("        Extension:  T2DeliverySystem:  plp_id = {}, t2_system_id = {}, siso_miso = {}, bandwidth = {}, \
                             guard_interval = {:?}, transmission_mode = {:?}, other_frequency = {}, tfs = {}, cells = {}",
                                         &t2_delivery_system.get_plp_id(),
                                         &t2_delivery_system.get_t2_system_id(),
                                         &t2_delivery_system.get_siso_miso(),
                                         &t2_delivery_system.get_bandwidth(),
                                         &t2_delivery_system.get_guard_interval(),
                                         &t2_delivery_system.get_transmission_mode(),
                                         &t2_delivery_system.get_other_frequency(),
                                         &t2_delivery_system.get_tfs(),
                                         "", // &t2_delivery_system.get_cells(),
                                );
                            }
                        },
                        x => println!("************  Got an extended descriptor type {:?}", x),
                    }
                },
                gst_mpegts::DVBDescriptorType::PrivateDataSpecifier => {
                    // It seems that this is the original_network_id being presented at the NIT section level.
                    let private_data = descriptor.parse_dvb_private_data_specifier().unwrap();
                    if PRINT_NIT {
                        println!("    Private Data Specifier: {}, {:?}", &private_data.0, &private_data.1);
                    }
                },
                x => {
                    // Get a lot of stream descriptor tag value 131 on Freeview. This is most
                    // likely a gst_mpegts::MiscDescriptorType::DtgLogicalChannel.
                    // There is also gst_mpets::ScteStreamType::IsochData and
                    // gst_mpegts::ATSCDescriptorType::Ac3 but it is seriously
                    // unlikely to be the latter on a DVB-T/DVB-T2 broadcast.
                    if x == gst_mpegts::DVBDescriptorType::__Unknown(131) {
                        let tag: gst_mpegts::MiscDescriptorType = unsafe { from_glib(x.to_glib()) };
                        assert_eq!(tag, gst_mpegts::MiscDescriptorType::DtgLogicalChannel);
                        let dtg_logical_channel_descriptor = descriptor.parse_logical_channel().unwrap();
                        if PRINT_NIT {
                            println!("    LogicalChannelDescriptor:");
                        }
                        for item in dtg_logical_channel_descriptor.get_channels().iter() {
                            if ! add_logical_channel_number_for_service_id(item.get_service_id(), item.get_logical_channel_number(), Some(&to_cw)) {
                                if PRINT_NIT {
                                    println!("Failed to add logical_channel_number {} to service_id {}.", &item.get_logical_channel_number(), &item.get_service_id());
                                }
                            }
                            if PRINT_NIT {
                                println!("        LogicalChannel:  service_id = {}, visible_service = {}, logical_channel_number = {}",
                                    &item.get_service_id(),
                                    &item.get_visible_service(),
                                    &item.get_logical_channel_number(),
                                );
                            }
                        }
                    } else {
                        println!("************  Got an unknown stream type {:?}", x)
                    }
                },
            }
            if PRINT_DATA {
                println!("            {:?}", &descriptor.get_data());
            }
        }
    }
}

fn build_pat(pat: &Vec<gst_mpegts::PatProgram>, to_cw: &glib::Sender<Message>) {
    // Only seem to get a couple of these on BBC News on Freeview from Crystal Palace.
    if PRINT_PAT {
        println!("========  PAT Section.");
    }
    for p in pat.iter() {
        if PRINT_PAT {
            println!("    PatProgram:  {}, {}", &p.get_program_number(), &p.get_network_or_program_map_pid());
        }
    }
}

fn build_pmt(pmt: &gst_mpegts::PMT, to_cw: &glib::Sender<Message>) {
    if PRINT_PMT {
        println!("========  PMT section:  program_number = {}", &pmt.get_program_number());
        for descriptor in pmt.get_descriptors().iter() {
            match descriptor.get_tag() {
                x => println!("************  Got an unhandled descriptor type {:?}", x)
            }
            if PRINT_DATA {
                println!("        {:?}", descriptor.get_data());
            }
        }
        for stream in pmt.get_streams().iter() {
            println!("    PMTStream:  stream_type = {:?}, pid = {}", stream.get_stream_type(), stream.get_pid());
            for descriptor in stream.get_descriptors().iter() {
                match descriptor.get_tag() {
                    gst_mpegts::DVBDescriptorType::Extension => {
                        match descriptor.get_tag_extension().unwrap() {
                            gst_mpegts::DVBExtendedDescriptorType::SupplementaryAudio => {
                                let supplementary_audio_extended_descriptor = descriptor.parse_supplementary_audio().unwrap();
                                println!("    Extension:  SupplementaryAudio: mix_type = {}, editorial_classification = {}, iso_639_language_code = {:?}, private_data = {:?}",
                                         &supplementary_audio_extended_descriptor.get_mix_type(),
                                         &supplementary_audio_extended_descriptor.get_editorial_classification(),
                                         &supplementary_audio_extended_descriptor.get_iso_639_language_code(),
                                         &supplementary_audio_extended_descriptor.get_private_data(),
                                );
                            },
                            x => println!("************  Got an unhandled extension descriptor type {:?}", x)
                        }
                    },
                    gst_mpegts::DVBDescriptorType::StreamIdentifier => {
                        let identifier = descriptor.parse_dvb_stream_identifier();
                        println!("        StreamIdentifier:  {:?}", &identifier);
                    },
                    gst_mpegts::DVBDescriptorType::Subtitling => {
                        let subtitling_descriptor = descriptor.parse_dvb_subtitling().unwrap();
                        for item in subtitling_descriptor.get_items().iter() {
                            println!("        Subtitling:  iso_639_language_code = {}, subtitling_type = {}, composition_page_id = {}, ancilliary_page_id = {}",
                                 &item.get_iso_639_language_code(),
                                 &item.get_subtitling_type(),
                                 &item.get_composition_page_id(),
                                 &item.get_ancillary_page_id());
                        }
                    },
                    gst_mpegts::DVBDescriptorType::DataBroadcastId => {
                        let (data_broadcast_id, id_selector_bytes) = descriptor.parse_dvb_data_broadcast_id().unwrap();
                        println!("    DataBroadcastId:  {}, {:?}", &data_broadcast_id, id_selector_bytes);
                    },
                    gst_mpegts::DVBDescriptorType::ApplicationSignalling => {
                        let data = descriptor.parse_application_signalling().unwrap();
                        println!("    ApplicationSignalling:  {:?}", &data);
                    },
                    x => {
                        let tag: gst_mpegts::DescriptorType = unsafe { from_glib(x.to_glib() as i32) };
                        match tag {
                            gst_mpegts::DescriptorType::Iso639Language => {
                                let language_descriptor = descriptor.parse_iso_639_language().unwrap();
                                println!("    Iso639Language: {:?}", &language_descriptor.get_items());
                            },
                            gst_mpegts::DescriptorType::DsmccCarouselIdentifier => {
                                // TODO Sort this out.
                                println!("    DsmccCarouselIdentifier: {:?}", &descriptor);
                            },
                            y => println!("************  Got an unhandled PMTStream descriptor type {:?}, {:?}", &x, &y)
                        }
                    }
                }
                if PRINT_DATA {
                    println!("            {:?}", &descriptor.get_data());
                }
            }
        }
    }
}

fn build_sdt(sdt: &gst_mpegts::SDT, to_cw: &glib::Sender<Message>) {
    if PRINT_SDT {
        println!("========  SDT section:  original_network_id = {:?}, transport_stream_id ={:?}", &sdt.get_original_network_id(), &sdt.get_transport_stream_id());
    }
    for service in sdt.get_services().iter() {
        if PRINT_SDT {
            println!("    SDTService:  service_id = {}, \
eit_schedule_flag = {}, \
eit_present_following = {}, \
running_status = {:?}, \
free_ca_mode = {}",
                     service.get_service_id(),
                     service.get_eit_schedule_flag(),
                     service.get_eit_present_following_flag(),
                     service.get_running_status(),
                     service.get_free_ca_mode(),
            );
        }
        for descriptor in service.get_descriptors().iter() {
            match descriptor.get_tag() {
                gst_mpegts::DVBDescriptorType::DefaultAuthority => {
                    let authority = descriptor.parse_dvb_default_authority();
                    if PRINT_SDT {
                        println!("        DefaultAuthority:  {:?}", authority);
                    }
                },
                // TODO Process Extension descriptors.
                gst_mpegts::DVBDescriptorType::Extension => {
                    match descriptor.get_tag_extension().unwrap() {
                        gst_mpegts::DVBExtendedDescriptorType::ServiceRelocated => {
                            let (old_original_network_id, old_transport_stream_id, old_service_id) =
                                descriptor.parse_dvb_service_relocated().unwrap();
                                if PRINT_SDT {
                                    println!("        Extension:  ServiceRelocated:  old_original_network_id = {}, old_transport_stream_id = {}, old_service_id = {}",
                                             old_original_network_id,
                                             old_transport_stream_id,
                                             old_service_id);
                                }
                        },
                        x => println!("************  Got an extended descriptor type {:?}", x),
                    }
                },
                gst_mpegts::DVBDescriptorType::FtaContentManagement => {
                    match descriptor.parse_fta_content_management() {
                        Some(d) =>
                            if PRINT_SDT {
                                println!("        FtaContentManagement:  user_defined = {}, do_not_scramble {}, control_remote_access_over_internet {:?}, do_not_apply_revocation = {}",
                                         d.get_user_defined(),
                                         d.get_do_not_scramble(),
                                         d.get_control_remote_access_over_internet(),
                                         d.get_do_not_apply_revocation(),
                                )
                            },
                        None => println!("        FtaContentManagement:  None"),
                    };
                },
                    gst_mpegts::DVBDescriptorType::PrivateDataSpecifier => {
                        let private_data = descriptor.parse_dvb_private_data_specifier().unwrap();
                        if PRINT_SDT {
                            println!("        PrivateDataSpecifier: {}, {:?}", &private_data.0, &private_data.1);
                        }
                    },
                gst_mpegts::DVBDescriptorType::Service => {
                    let (service_type, service_name, some_string_possibly_empty) = descriptor.parse_dvb_service().unwrap();
                    if PRINT_SDT {
                        println!("        Service:  {:?}, '{}', '{}'", service_type, service_name, some_string_possibly_empty);
                    }
                },
                x => println!("************  Got an unhandled descriptor of type {:?}", x)
            }
            if PRINT_DATA {
                println!("            {:?}", descriptor.get_data());
            }
        }
    }
}

fn build_tdt(tdt: &gst::DateTime, to_cw: &glib::Sender<Message>) {
    if PRINT_TDT {
        println!("========  TDT section:  utc_time = {}", &tdt);
    }
}

fn build_tsdt(tsdt: &Vec<gst_mpegts::Descriptor>, to_cw: &glib::Sender<Message>) {
    // Do not seem to get any of these on BBC News on Freeview from Crystal Palace.
    if PRINT_TSDT {
        println!("========  TSDT section:  {:?}", &tsdt);
    }
}

fn build_tot(tot: &gst_mpegts::TOT, to_cw: &glib::Sender<Message>) {
    if PRINT_TOT {
        println!("========  TOT section:  utc_time = {}", &tot.get_utc_time());
    }
    for descriptor in tot.get_descriptors().iter() {
        match descriptor.get_tag() {
            gst_mpegts::DVBDescriptorType::LocalTimeOffset => {
                let local_time_offset = descriptor.parse_local_time_offset().unwrap();
                if PRINT_TOT {
                    println!("    LocalTimeOffset:");
                }
                for item in local_time_offset.get_items().iter() {
                    if PRINT_TOT {
                        println!("        LocalTimeOffsetItem:  \
                        country_code = {}, \
                        country_region_id = {}, \
                        local_time_offset_polarity = {}, \
                        local_time_offset = {}, \
                        time_of_change = {}, \
                        next_time_offset = {}",
                                 &item.get_country_code(),
                                 &item.get_country_region_id(),
                                 &item.get_local_time_offset_polarity(),
                                 &item.get_local_time_offset(),
                                 &item.get_time_of_change(),
                                 &item.get_next_time_offset(),
                        );
                    }
                }
            },
            x => println!("************  Got an unhandled descriptor of type {:?}", x)
        }
        if PRINT_DATA {
            println!("        {:?}", &descriptor.get_data());
        }
    }
}

/// The main dæmon for EPG management.
///
/// Process the [Section](struct.Section.html) instances sent on the `from_gstreamer` channel.
///
/// This is a separate process executed by a thread other than the Glib event loop thread
/// so as to avoid that thread having to do too much work.
pub fn run(to_cw: glib::Sender<Message>, from_gstreamer: std::sync::mpsc::Receiver<gst_mpegts::Section>) {
    loop {
        match from_gstreamer.recv() {
            Ok(mut section) => {
                match section.get_section_type() {
                    gst_mpegts::SectionType::AtscCvct => {},
                    gst_mpegts::SectionType::AtscEit => {},
                    gst_mpegts::SectionType::AtscEtt => {},
                    gst_mpegts::SectionType::AtscMgt => {},
                    gst_mpegts::SectionType::AtscStt => {},
                    gst_mpegts::SectionType::AtscTvct => {},
                    gst_mpegts::SectionType::Bat => {
                        if let Some(bat) = section.get_bat() {
                            build_bat(&bat, &to_cw);
                        } else {
                            println!("******** Got a BAT that wasn't a BAT {:?}", &section);
                        }
                    },
                    gst_mpegts::SectionType::Cat => {
                        build_cat(&section.get_cat(), &to_cw);
                    },
                    gst_mpegts::SectionType::Eit => {
                        if let Some(eit) = section.get_eit() {
                            build_eit(&eit, &to_cw);
                        } else {
                            println!("********  Got an EIT that wasn't an EIT {:?}", &section);
                            println!("********      Section type: {:?}", &section.get_section_type());
                            println!("********      EIT: {:?}", &section.get_eit());
                        }
                    },
                    gst_mpegts::SectionType::Nit => {
                        if let Some(nit) = section.get_nit() {
                            build_nit(&nit, &to_cw);
                        } else {
                            println!("******** Got a NIT that wasn't a NIT {:?}", &section);
                        }
                    },
                    gst_mpegts::SectionType::Pat => {
                        build_pat(&section.get_pat(), &to_cw);
                    },
                    gst_mpegts::SectionType::Pmt => {
                        if let Some(pmt) = section.get_pmt() {
                            build_pmt(&pmt, &to_cw);
                        } else {
                            println!("******** Got a PMT that wasn't a PMT {:?}", &section);
                        }
                    },
                    gst_mpegts::SectionType::Sdt => {
                        if let Some(sdt) = section.get_sdt() {
                            build_sdt(&sdt, &to_cw);
                        } else {
                            println!("******** Got a SDT that wasn't a SDT {:?}", &section);
                        }
                    },
                    gst_mpegts::SectionType::Tdt => {
                        if let Some(tdt) = section.get_tdt() {
                            build_tdt(&tdt, &to_cw);
                        }else {
                            println!("******** Got a TDT that wasn't a TDT {:?}", &section);
                        }
                    },
                    gst_mpegts::SectionType::Tsdt => {
                        build_tsdt(&section.get_tsdt(), &to_cw);
                    },
                    gst_mpegts::SectionType::Tot => {
                        if let Some(tot) = section.get_tot() {
                            build_tot(&tot, &to_cw);
                        } else {
                            println!("******** Got a TOT that wasn't a TOT {:?}", &section);
                        }
                    },
                    gst_mpegts::SectionType::Unknown => {
                        println!("======== Got an Unknown section.");
                    },
                    x => {
                        println!("******** got an unknown section type, number {:?}", x);
                    },
                }
            },
            Err(e) => {
                println!("********  failed to receive a section {:?}", e);
            }
        }
    }
}
