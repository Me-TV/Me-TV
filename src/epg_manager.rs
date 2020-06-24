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
use glib::translate::{ToGlib};

use gst;
use gst_mpegts;

use crate::control_window::Message;

fn build_bat(bat: &gst_mpegts::BAT) {
    /*
    println!("======== Got a BAT section.");
    for descriptor in bat.get_descriptors().iter() {
        println!("         {:?}", descriptor);
    }
    println!("========");
    for stream in bat.get_streams().iter() {
        println!("         {:?}", stream);
    }
     */
}

fn build_cat(cat: &Vec<gst_mpegts::Descriptor>) {
    /*
    if cat.len() > 0 {
        println!("======== Got a non-empty CAT section {:?}", &cat);
    }
     */
}

fn build_eit(eit: &gst_mpegts::EIT) {
    /*
    println!("======== Got an EIT section");
    for event in eit.get_events().iter() {
        println!("    event_id = {:?}", event.get_event_id());
        for d in event.get_descriptors().iter() {
            println!("        {:?}: {:?}", d.get_tag(), d.get_data());
            match d.get_tag() {
                gst_mpegts::DVBDescriptorType::Component => {
                    let component = d.parse_dvb_component().unwrap();
                    println!("            {:?}", &component);
                },
                gst_mpegts::DVBDescriptorType::Content => {
                    let c = d.parse_dvb_content().unwrap();
                    for item in c.iter() {
                        println!("            {}", gst_mpegts::content_description(item.get_content_nibble_1().to_glib(), item.get_content_nibble_2()))
                    }
                },
                gst_mpegts::DVBDescriptorType::ContentIdentifier => {
                    println!("            Unknown processing technique");
                },
                gst_mpegts::DVBDescriptorType::Linkage => {
                    let linkage = d.parse_dvb_linkage().unwrap();
                    println!("            {:?}", &linkage);
                }
                gst_mpegts::DVBDescriptorType::ShortEvent =>{
                    // TODO It seems this can panic
                    //   assertion failed: !ptr.is_null()', /home/users/russel/.cargo/git/checkouts/glib-928cf7b282977403/3a64675/src/gstring.rs:51:9
                    //   on real data from Freeview.
                    //
                    // See https://gitlab.freedesktop.org/gstreamer/gst-plugins-bad/-/issues/1333
                    match panic::catch_unwind(|| {
                        let (language_code, title, blurb) = d.parse_dvb_short_event().unwrap();
                        println!("            {}, {:?}, {:?}", &language_code, &title, &blurb);
                    }) {
                        Ok(_) => {},
                        Err(_) => println!("****************  parse_dvb_short_event paniced, assume there is a 0x1f encoding byte in the string."),
                    }
                },
                gst_mpegts::DVBDescriptorType::PrivateDataSpecifier => {
                    println!("            {:?}", &d.parse_dvb_private_data_specifier());
                },
                gst_mpegts::DVBDescriptorType::FtaContentManagement => {
                    println!("            Unknown processing technique");
                },
                x => println!("Unprocessed tag: {:?}", x),
            }
        }
    }
     */
}

fn build_nit(nit: &gst_mpegts::NIT) {
    println!("======== NIT section: actual_network = {}, network_id = {}", nit.get_actual_network(), nit.get_network_id());
    for descriptor in nit.get_descriptors().iter() {
        // EN 300 468 Table 12 states which descriptors are allowed.
        match descriptor.get_tag() {
            gst_mpegts::DVBDescriptorType::NetworkName => {
                let name = descriptor.parse_dvb_network_name().unwrap();
                println!("    Network Name:  {}", &name);
            },
            gst_mpegts::DVBDescriptorType::Extension => {
                match descriptor.get_tag_extension().unwrap() {
                    gst_mpegts::DVBExtendedDescriptorType::TargetRegionName => {
                        let target_region_name = descriptor.parse_target_region_name().unwrap();
                        println!("    Extension:  TargetRegionName:   country_code = {}, iso_639_language_code = {}, region_data = {:?}",
                                 &target_region_name.get_country_code(),
                                 &target_region_name.get_iso_639_language_code(),
                                 &target_region_name.get_region_data());
                    },
                    gst_mpegts::DVBExtendedDescriptorType::TargetRegion => {
                        let target_region = &descriptor.parse_target_region().unwrap();
                        println!("    Extension:  TargetRegion:  country_code = {}, additional_country_codes = {:?}",
                                 &target_region.get_country_code(),
                                 &target_region.get_additional_country_codes());
                    },
                    gst_mpegts::DVBExtendedDescriptorType::Message => {
                        let message = descriptor.parse_message().unwrap();
                        println!("    Extension:  Message:  message_id = {}, iso_639_language_code = {}, message = {}",
                                 &message.get_message_id(),
                                 &message.get_iso_639_language_code(),
                                 &message.get_message());
                    },
                    gst_mpegts::DVBExtendedDescriptorType::UriLinkage => {
                        let uri_linkage = descriptor.parse_uri_linkage().unwrap();
                        println!("    Extension:  UriLinkage:  uri_linkage_type = {:?}, uri = {}, min_polling_interval = {}, private_data = {:?}",
                                 &uri_linkage.get_uri_linkage_type(),
                                 &uri_linkage.get_uri(),
                                 &uri_linkage.get_min_polling_interval(),
                                 &uri_linkage.get_private_data());
                    },
                    x => println!("********  Got an extended descriptor type {:?}", x),
                }
            },
            gst_mpegts::DVBDescriptorType::Linkage => {
                let linkage = descriptor.parse_dvb_linkage().unwrap();
                println!("    Linkage:  transport_stream_id = {}, original_network_id = {}, service_id = {}, linkage_type = {:?}",
                         linkage.get_transport_stream_id(),
                         linkage.get_original_network_id(),
                         linkage.get_service_id(),
                         linkage.get_linkage_type());
                // TODO get the event, extended event, mobile hand over
                // TODO get private data.
            },
            gst_mpegts::DVBDescriptorType::PrivateDataSpecifier => {
                // It seems that this is the original_network_id being presented at the NIT section level.
                let private_data = descriptor.parse_dvb_private_data_specifier().unwrap();
                println!("    Private Data Specifier: {}, {:?}", &private_data.0, &private_data.1);
            },
            x => println!("********  Got a not allowed descriptor type {:?}", x),
        }
        println!("        {:?}", descriptor.get_data());
    }
    for stream in nit.get_streams().iter() {
        println!("    NITStream:  transport_stream_id = {}, original_network_id = {}", stream.get_transport_stream_id(), stream.get_original_network_id());
        for descriptor in stream.get_descriptors().iter() {
            match descriptor.get_tag() {
                gst_mpegts::DVBDescriptorType::ServiceList => {
                    let service_list = descriptor.parse_dvb_service_list().unwrap();
                    println!("        Service List:");
                    for service in service_list.iter() {
                        println!("            service_id = {}, service_type = {:?}",
                                 &service.get_service_id(),
                                 &service.get_type())
                    }
                },
                gst_mpegts::DVBDescriptorType::TerrestrialDeliverySystem => {
                    let terrestrial_delivery_system = descriptor.parse_terrestrial_delivery_system().unwrap();
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
                },
                gst_mpegts::DVBDescriptorType::Extension => {
                    match descriptor.get_tag_extension().unwrap() {
                        gst_mpegts::DVBExtendedDescriptorType::TargetRegion => {
                            let target_region = descriptor.parse_target_region().unwrap();
                            println!("        Extension:  TargetRegion:  country_code = {}, additional_country_codes = {:?}",
                                     &target_region.get_country_code(),
                                     &target_region.get_additional_country_codes());
                        },
                        gst_mpegts::DVBExtendedDescriptorType::T2DeliverySystem => {
                            let t2_delivery_system = descriptor.parse_dvb_t2_delivery_system().unwrap();
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
                        },
                        x => println!("********  Got an extended descriptor type {:?}", x),
                    }
                },
                gst_mpegts::DVBDescriptorType::PrivateDataSpecifier => {
                    // It seems that this is the original_network_id being presented at the NIT section level.
                    let private_data = descriptor.parse_dvb_private_data_specifier().unwrap();
                    println!("    Private Data Specifier: {}, {:?}", &private_data.0, &private_data.1);
                },
                x => {
                    // Get a lot of stream descriptor tag value 131 on Freeview. This is most
                    // likely a gst_mpets::ScteStreamType::IsochData though there are also
                    // gst_mpegts::MiscDescriptorType::DtgLogicalChannel and
                    // gst_mpegts::ATSCDescriptorType::Ac3 as tags that have the value 131.
                    if x == gst_mpegts::DVBDescriptorType::__Unknown(131) {
                        // TODO Do something sensible here.
                        println!("********  Got a stream with tag 131, not sure how to process it.");
                    } else {
                        println!("********  Got an unknown stream type {:?}", x)
                    }
                },
            }
            println!("            {:?}", &descriptor.get_data());
        }
    }
}

fn build_pat(pat: &Vec<gst_mpegts::PatProgram>) {
    /*
    for p in pat.iter() {
        println!("======== {:?}", &p);
    }
     */
}

fn build_pmt(pmt: &gst_mpegts::PMT) {
    //println!("======== Got a PMT section {:?}, {:?}, {:?}", &pmt.get_program_number(), &pmt.get_descriptors(), &pmt.get_streams());
}

fn build_sdt(sdt: &gst_mpegts::SDT) {
    //println!("======== Got a SDT section {:?}, {:?}, {:?}", &sdt.get_original_network_id(), &sdt.get_transport_stream_id(), &sdt.get_services());
}

fn build_tdt(tdt: &gst_mpegts::Section) {
    //println!("======== Got a TDT section {:?}", &tdt);
}

fn build_tsdt(tsdt: &Vec<gst_mpegts::Descriptor>) {
    //println!("======== Got a TSDT section {:?}", &tsdt);
}

fn build_tot(tot: &gst_mpegts::TOT) {
    //println!("======== Got a TOT section {:?}, {:?}", &tot.get_utc_time(), &tot.get_descriptors());
}

/// The main dæmon for EPG management.
///
/// Process the `gst_mpegts::Section` instances sent on the `from_gstreamer` channel.
///
/// This is a separate process executed by a thread other than the Glib event loop thread
/// so as to avoid that thread having to do too much work.
pub fn run(mut to_cw: glib::Sender<Message>, from_gstreamer: std::sync::mpsc::Receiver<gst_mpegts::Section>) {
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
                            build_bat(&bat);
                        } else {
                            println!("******** Got a BAT that wasn't a BAT {:?}", &section);
                        }
                    },
                    gst_mpegts::SectionType::Cat => {
                        build_cat(&section.get_cat());
                    },
                    gst_mpegts::SectionType::Eit => {
                        if let Some(eit) = section.get_eit() {
                            build_eit(&eit);
                        } else {
                            println!("********  Got an EIT that wasn't an EIT {:?}", &section);
                            println!("********      Section type: {:?}", &section.get_section_type());
                            println!("********      EIT: {:?}", &section.get_eit());
                        }
                    },
                    gst_mpegts::SectionType::Nit => {
                        if let Some(nit) = section.get_nit() {
                            build_nit(&nit);
                        } else {
                            println!("******** Got a NIT that wasn't a NIT {:?}", &section);
                        }
                    },
                    gst_mpegts::SectionType::Pat => {
                        build_pat(&section.get_pat());
                    },
                    gst_mpegts::SectionType::Pmt => {
                        if let Some(pmt) = section.get_pmt() {
                            build_pmt(&pmt);
                        } else {
                            println!("******** Got a PMT that wasn't a PMT {:?}", &section);
                        }
                    },
                    gst_mpegts::SectionType::Sdt => {
                        if let Some(sdt) = section.get_sdt() {
                            build_sdt(&sdt);
                        } else {
                            println!("******** Got a SDT that wasn't a SDT {:?}", &section);
                        }
                    },
                    gst_mpegts::SectionType::Tdt => {
                        build_tdt(&section);
                    },
                    gst_mpegts::SectionType::Tsdt => {
                        build_tsdt(&section.get_tsdt());
                    },
                    gst_mpegts::SectionType::Tot => {
                        if let Some(tot) = section.get_tot() {
                            build_tot(&tot);
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
