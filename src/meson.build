config_data = configuration_data()
config_data.set_quoted('NAME', name)
config_data.set_quoted('AUTHORS', authors)
config_data.set_quoted('APP_ID', app_id)
config_data.set_quoted('PKGDATADIR', pkgdatadir)
config_data.set_quoted('VERSION', version)

config_source = configure_file(
    configuration: config_data,
    input: 'config.rs.in',
    output: 'config.rs'
)

run_command(
    'cp',
    join_paths(meson.build_root(), 'src', 'config.rs'),
    join_paths(meson.source_root(), 'src', 'config.rs'),
    check: true
)

sources = files(
    'config.rs',
    'main.rs'
)

# Run cargo
cargo_build_script = find_program(join_paths(meson.source_root(), 'build-aux/cargo.sh'))
custom_target(
    'cargo-build',
    build_by_default: true,
    input: sources,
    output: meson.project_name(),
    console: true,
    install: true,
    install_dir: get_option('bindir'),
    command: [
        cargo_build_script,
        meson.build_root(),
        meson.source_root(),
        '@OUTPUT@',
        get_option('buildtype'),
        meson.project_name()
    ]
)