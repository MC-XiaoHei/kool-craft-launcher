const { name, version, identifier } = readPackageJson()

updateTauriConfig()
updateCargoToml()

console.log('Tauri config updated with package.json values.')

function readPackageJson() {
  const packageJsonFile = resolve('package.json')
  const packageJson = JSON.parse(readFile(packageJsonFile))

  return {
    name: packageJson.name,
    version: packageJson.version,
    identifier: packageJson.identifier
  }
}

function updateTauriConfig() {
  const tauriConfigFile = resolve('src-tauri/tauri.conf.json')
  const tauriConfig = JSON.parse(readFile(tauriConfigFile))

  tauriConfig.productName = name
  tauriConfig.version = version
  tauriConfig.identifier = identifier

  writeFile(tauriConfigFile, JSON.stringify(tauriConfig, null, 2) + '\n')
}

function updateCargoToml() {
  const cargoTomlFile = resolve('src-tauri/Cargo.toml')
  const cargoToml = readFile(cargoTomlFile, 'utf8')

  const updatedCargoToml = cargoToml
    .replace(/^name\s*=\s*".*"/m, `name = "${name}"`)
    .replace(/^version\s*=\s*".*"/m, `version = "${version}"`)

  writeFile(cargoTomlFile, updatedCargoToml)
}

function resolve(file) {
  return require('path').resolve(__dirname, file)
}

function readFile(path) {
  return require('fs').readFileSync(path, 'utf8')
}

function writeFile(path, content) {
  require('fs').writeFileSync(path, content)
}