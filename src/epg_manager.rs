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

use glib;

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
    //println!("======== Got an EIT section {:?}", &eit.get_events());
}

fn build_nit(nit: &gst_mpegts::NIT) {
    println!("======== NIT section");
    for descriptor in nit.get_descriptors().iter() {
        match descriptor.get_tag() {
            gst_mpegts::DVBDescriptorType::NetworkName => println!("    Network Name: {}", descriptor.parse_dvb_network_name().unwrap()),
            gst_mpegts::DVBDescriptorType::Extension => println!("    Extension {}, {:?}", descriptor.get_tag_extension().unwrap(), descriptor.get_data()),
            gst_mpegts::DVBDescriptorType::Linkage => println!("    Linkage: {:?}", descriptor.parse_dvb_linkage().unwrap()),
            gst_mpegts::DVBDescriptorType::PrivateDataSpecifier => println!("    Private Data Specifier: {:?}", descriptor.parse_dvb_private_data_specifier().unwrap()),
            _ => println!("WTF"),
        }
    }
    for stream in nit.get_streams().iter() {
        println!("   transport_stream_id = {}, original_network_id = {}", stream.get_transport_stream_id(), stream.get_original_network_id());
        for d in stream.get_descriptors().iter() {
            match d.get_tag() {
                gst_mpegts::DVBDescriptorType::ServiceList => {
                    println!("    Service List");
                    for item in d.parse_dvb_service_list().unwrap().iter() {
                        println!("        {:?}", item);
                    }
                },
                gst_mpegts::DVBDescriptorType::TerrestrialDeliverySystem => println!("    TerrestrialDeliverySystem {:?}", d.parse_terrestrial_delivery_system().unwrap()),
                gst_mpegts::DVBDescriptorType::Extension => println!("    Extension {}", d.get_tag_extension().unwrap()),
                _ => println!("WTF"),
            }
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
