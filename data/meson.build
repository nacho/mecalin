# Desktop file
desktop_file = i18n.merge_file(
  input: 'io.github.nacho.teclazo.desktop.in',
  output: 'io.github.nacho.teclazo.desktop',
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: get_option('datadir') / 'applications',
)

# GSchema
install_data('io.github.nacho.teclazo.gschema.xml',
  install_dir: get_option('datadir') / 'glib-2.0' / 'schemas'
)

# Metainfo
install_data('io.github.nacho.teclazo.metainfo.xml',
  install_dir: get_option('datadir') / 'metainfo'
)

# Compile GSettings schemas
gnome.compile_schemas(build_by_default: true)

subdir('icons')
