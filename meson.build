# -*- mode:python; coding:utf-8; -*-

project(
    'Me TV',
    'rust',
    version: '3.0.4',
    license: 'GPLv3',
)

prefix = get_option('prefix')
bindir = get_option('bindir')
datadir = get_option('datadir')
applicationsdir = join_paths(datadir, 'applications')
pixmapsdir = join_paths(datadir, 'pixmaps')

cargo = find_program('cargo')
cargo_script = find_program('scripts/cargo.sh')
cargo_test_script = find_program('scripts/cargo_test.sh')

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
    command: [cargo_script, '@CURRENT_SOURCE_DIR@', '@OUTPUT@']
)

conf_data = configuration_data()
conf_data.set('PREFIX', prefix)
configure_file(
    input: 'data/me-tv.desktop.in',
    output: 'me-tv.desktop',
    configuration: conf_data,
    install_dir: applicationsdir,
)

install_data('src/resources/images/me-tv.png', install_dir: pixmapsdir)
install_man('doc/me-tv.1')

me_tv_test_name = me_tv_target_name + '-test'

me_tv_test = custom_target(
    me_tv_test_name,
    build_by_default: false,
    console: true,
    input: sources,
    output: [me_tv_test_name],
    command: [cargo_test_script, '@CURRENT_SOURCE_DIR@', '@OUTPUT@'],
)

# Apparently it is not possible to use a custom_target in a test. :-(

custom_target(
    'run-test',
    output: 'run-test',
    command: me_tv_test.full_path(),
)
