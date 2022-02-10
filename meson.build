project(
    'metanote',
    'rust',
    version: '0.1.0',
    license: 'GPL-3.0-or-later',
    meson_version: '>= 0.50.0'
)

# Project-wide variables
name        = 'Metanote'
app_id      = 'com.github.bmreading.Metanote'
version     = meson.project_version()
authors     = 'Brian Reading <brian.reading@gmail.com>' # Separate authors by commas in string
datadir     = join_paths(get_option('prefix'), get_option('datadir'))
localedir   = join_paths(get_option('prefix'), get_option('localedir'))
pkgdatadir  = join_paths(datadir, meson.project_name())

# Import modules
gnome = import('gnome')

# Subdirectories
subdir('data')
subdir('src')
