/* tslint:disable */
/* eslint-disable */
/* prettier-ignore */

/* auto-generated by NAPI-RS */

const { existsSync, readFileSync } = require('fs')
const { join } = require('path')

const { platform, arch } = process

let nativeBinding = null
let localFileExisted = false
let loadError = null

function isMusl() {
  // For Node 10
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      const lddPath = require('child_process').execSync('which ldd').toString().trim()
      return readFileSync(lddPath, 'utf8').includes('musl')
    } catch (e) {
      return true
    }
  } else {
    const { glibcVersionRuntime } = process.report.getReport().header
    return !glibcVersionRuntime
  }
}

switch (platform) {
  case 'android':
    switch (arch) {
      case 'arm64':
        localFileExisted = existsSync(join(__dirname, 'libgit.android-arm64.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./libgit.android-arm64.node')
          } else {
            nativeBinding = require('@alekitto/libgit-android-arm64')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm':
        localFileExisted = existsSync(join(__dirname, 'libgit.android-arm-eabi.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./libgit.android-arm-eabi.node')
          } else {
            nativeBinding = require('@alekitto/libgit-android-arm-eabi')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Android ${arch}`)
    }
    break
  case 'win32':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(
          join(__dirname, 'libgit.win32-x64-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./libgit.win32-x64-msvc.node')
          } else {
            nativeBinding = require('@alekitto/libgit-win32-x64-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'ia32':
        localFileExisted = existsSync(
          join(__dirname, 'libgit.win32-ia32-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./libgit.win32-ia32-msvc.node')
          } else {
            nativeBinding = require('@alekitto/libgit-win32-ia32-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'libgit.win32-arm64-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./libgit.win32-arm64-msvc.node')
          } else {
            nativeBinding = require('@alekitto/libgit-win32-arm64-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Windows: ${arch}`)
    }
    break
  case 'darwin':
    localFileExisted = existsSync(join(__dirname, 'libgit.darwin-universal.node'))
    try {
      if (localFileExisted) {
        nativeBinding = require('./libgit.darwin-universal.node')
      } else {
        nativeBinding = require('@alekitto/libgit-darwin-universal')
      }
      break
    } catch {}
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(join(__dirname, 'libgit.darwin-x64.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./libgit.darwin-x64.node')
          } else {
            nativeBinding = require('@alekitto/libgit-darwin-x64')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'libgit.darwin-arm64.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./libgit.darwin-arm64.node')
          } else {
            nativeBinding = require('@alekitto/libgit-darwin-arm64')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on macOS: ${arch}`)
    }
    break
  case 'freebsd':
    if (arch !== 'x64') {
      throw new Error(`Unsupported architecture on FreeBSD: ${arch}`)
    }
    localFileExisted = existsSync(join(__dirname, 'libgit.freebsd-x64.node'))
    try {
      if (localFileExisted) {
        nativeBinding = require('./libgit.freebsd-x64.node')
      } else {
        nativeBinding = require('@alekitto/libgit-freebsd-x64')
      }
    } catch (e) {
      loadError = e
    }
    break
  case 'linux':
    switch (arch) {
      case 'x64':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'libgit.linux-x64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./libgit.linux-x64-musl.node')
            } else {
              nativeBinding = require('@alekitto/libgit-linux-x64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'libgit.linux-x64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./libgit.linux-x64-gnu.node')
            } else {
              nativeBinding = require('@alekitto/libgit-linux-x64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'arm64':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'libgit.linux-arm64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./libgit.linux-arm64-musl.node')
            } else {
              nativeBinding = require('@alekitto/libgit-linux-arm64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'libgit.linux-arm64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./libgit.linux-arm64-gnu.node')
            } else {
              nativeBinding = require('@alekitto/libgit-linux-arm64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'arm':
        localFileExisted = existsSync(
          join(__dirname, 'libgit.linux-arm-gnueabihf.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./libgit.linux-arm-gnueabihf.node')
          } else {
            nativeBinding = require('@alekitto/libgit-linux-arm-gnueabihf')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Linux: ${arch}`)
    }
    break
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`)
}

if (!nativeBinding) {
  if (loadError) {
    throw loadError
  }
  throw new Error(`Failed to load native binding`)
}

const { Commit, Time, Signature, Credentials, Index, Oid, Object, ReferenceType, Reference, Remote, RemoteHead, Repository, InitOptions, Revwalk, Tree, TreeEntry, RepositoryState, ResetType, Direction, Sort } = nativeBinding

module.exports.Commit = Commit
module.exports.Time = Time
module.exports.Signature = Signature
module.exports.Credentials = Credentials
module.exports.Index = Index
module.exports.Oid = Oid
module.exports.Object = Object
module.exports.ReferenceType = ReferenceType
module.exports.Reference = Reference
module.exports.Remote = Remote
module.exports.RemoteHead = RemoteHead
module.exports.Repository = Repository
module.exports.InitOptions = InitOptions
module.exports.Revwalk = Revwalk
module.exports.Tree = Tree
module.exports.TreeEntry = TreeEntry
module.exports.RepositoryState = RepositoryState
module.exports.ResetType = ResetType
module.exports.Direction = Direction
module.exports.Sort = Sort
