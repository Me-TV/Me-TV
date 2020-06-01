# -*- mode:python; coding:utf-8; -*-

project(
    'Me TV',
    'rust',
    #version: '', # This is not used due to the way things are built, so don't have it.
    default_options: ['buildtype=release'],
    license: 'GPLv3',
)

application_id = 'uk.org.winder.me-tv'

prefix = get_option('prefix')
bindir = get_option('bindir')
datadir = get_option('datadir')
applicationsdir = join_paths(datadir, 'applications')
iconsdir = join_paths(datadir, 'icons', 'hicolor', '48x48', 'apps')

dependency('glib-2.0')
dependency('gio-2.0')
dependency('gtk+-3.0')
dependency('gstreamer-1.0')
dependency('gstreamer-plugins-base-1.0')
dependency('gstreamer-plugins-bad-1.0')
dependency('gstreamer-mpegts-1.0')

cargo = find_program('cargo')
cargo_script = find_program('scripts/cargo.sh')

sources = run_command('sh', '-c', 'cd ${MESON_SOURCE_ROOT} && ls src/*.rs').stdout().strip().split()

me_tv_target_name = 'me-tv'

me_tv = custom_target(
    me_tv_target_name,
    build_by_default: true,
    console: true,
    input: sources,
    output: [me_tv_target_name],
    install: true,
    install_dir: bindir,
    command: [cargo_script, '@CURRENT_SOURCE_DIR@', '@OUTPUT@', get_option('buildtype'),]
)

me_tv_recording_target_name = me_tv_target_name + '-record'

me_tv_recording = custom_target(
    me_tv_recording_target_name,
    build_by_default: true,
    console: true,
    input: 'src/bin/' + me_tv_recording_target_name + '.rs',
    output: [me_tv_recording_target_name],
    install: true,
    install_dir: bindir,
    command: [cargo_script, '@CURRENT_SOURCE_DIR@', '@OUTPUT@', get_option('buildtype'),]
)

me_tv_schedule_target_name = me_tv_target_name + '-schedule'

me_tv_schedule = custom_target(
    me_tv_schedule_target_name,
    build_by_default: true,
    console: true,
    input: 'src/bin/' + me_tv_schedule_target_name + '.rs',
    output: [me_tv_schedule_target_name],
    install: true,
    install_dir: bindir,
    command: [cargo_script, '@CURRENT_SOURCE_DIR@', '@OUTPUT@', get_option('buildtype'),]
)

install_man('doc/me-tv.1')

install_data(
    'data/@0@.desktop'.format(application_id),
    install_dir: datadir / 'applications'
)

install_data(
    'data/@0@.metainfo.xml'.format(application_id),
    install_dir: datadir / 'metainfo'
)

install_data(
    'src/resources/images/@0@.png'.format(application_id),
    install_dir: datadir / 'icons' / 'hicolor' / '48x48' / 'apps'
)
