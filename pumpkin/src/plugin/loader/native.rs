use std::{
    any::Any,
    mem::{align_of, size_of},
    sync::LazyLock,
};

use libloading::Library;

use crate::plugin::{
    PLUGIN_API_VERSION,
    loader::{PluginLoadFuture, PluginUnloadFuture},
};

use super::{LoaderError, Path, Plugin, PluginLoader, PluginMetadata};

pub struct NativePluginLoader;

const fn validate_api_version(plugin_version: u32) -> Result<(), LoaderError> {
    if plugin_version == PLUGIN_API_VERSION {
        Ok(())
    } else {
        Err(LoaderError::ApiVersionMismatch {
            plugin_version,
            server_version: PLUGIN_API_VERSION,
        })
    }
}

const fn validate_metadata_layout(
    plugin_size: usize,
    plugin_align: usize,
) -> Result<(), LoaderError> {
    let server_size = size_of::<PluginMetadata>();
    let server_align = align_of::<PluginMetadata>();
    if plugin_size == server_size && plugin_align == server_align {
        Ok(())
    } else {
        Err(LoaderError::MetadataLayoutMismatch {
            plugin_size,
            plugin_align,
            server_size,
            server_align,
        })
    }
}

impl PluginLoader for NativePluginLoader {
    fn load<'a>(&'a self, path: &'a Path) -> PluginLoadFuture<'a> {
        Box::pin(async {
            let path = path.to_owned();

            let library = unsafe { Library::new(&path) }
                .map_err(|e| LoaderError::LibraryLoad(e.to_string()))?;

            // Ensure this plugin was built against a compatible Pumpkin plugin API version
            let plugin_api_version = unsafe {
                match library.get::<*const u32>(b"PUMPKIN_API_VERSION") {
                    Ok(symbol) => **symbol,
                    Err(_) => return Err(LoaderError::ApiVersionMissing),
                }
            };

            validate_api_version(plugin_api_version)?;

            // Validate the metadata layout before interpreting plugin-owned memory as a Rust type.
            let (plugin_metadata_size, plugin_metadata_align) = unsafe {
                let size = library
                    .get::<*const usize>(b"PUMPKIN_METADATA_SIZE")
                    .map_err(|_| LoaderError::MetadataLayoutMissing)?;
                let align = library
                    .get::<*const usize>(b"PUMPKIN_METADATA_ALIGN")
                    .map_err(|_| LoaderError::MetadataLayoutMissing)?;
                (**size, **align)
            };
            validate_metadata_layout(plugin_metadata_size, plugin_metadata_align)?;

            // 2. Extract Metadata (METADATA). The plugin macro exports a
            // `LazyLock<PluginMetadata>`, so initialize and dereference that wrapper before
            // cloning the metadata. Reading the symbol as a bare `PluginMetadata` interprets
            // the lazy initializer state as strings and vectors, which is undefined behavior.
            let metadata = unsafe {
                &***library
                    .get::<*const LazyLock<PluginMetadata>>(b"METADATA")
                    .map_err(|_| LoaderError::MetadataMissing)?
            };

            // 3. Extract Plugin Factory (plugin)
            let plugin_factory = unsafe {
                library
                    .get::<fn() -> Box<dyn Plugin>>(b"plugin")
                    .map_err(|_| LoaderError::EntrypointMissing)?
            };

            Ok((
                plugin_factory(),
                metadata.clone(),
                Box::new(library) as Box<dyn Any + Send + Sync>,
            ))
        })
    }

    fn can_load(&self, path: &Path) -> bool {
        let ext = path.extension().unwrap_or_default();

        if cfg!(target_os = "windows") {
            ext.eq_ignore_ascii_case("dll")
        } else if cfg!(target_os = "macos") {
            ext.eq_ignore_ascii_case("dylib")
        } else {
            ext.eq_ignore_ascii_case("so")
        }
    }

    fn unload(&self, data: Box<dyn Any + Send + Sync>) -> PluginUnloadFuture<'_> {
        Box::pin(async {
            data.downcast::<Library>()
                .map_or(Err(LoaderError::InvalidLoaderData), |library| {
                    drop(library);
                    Ok(())
                })
        })
    }

    /// Windows specific issue: Windows locks DLLs, so we must indicate they cannot be unloaded.
    fn can_unload(&self) -> bool {
        !cfg!(target_os = "windows")
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_api_version, validate_metadata_layout};
    use crate::plugin::{PLUGIN_API_VERSION, PluginMetadata, loader::LoaderError};
    use std::mem::{align_of, size_of};

    #[test]
    fn rejects_stale_native_plugin_api_before_reading_metadata() {
        let error = validate_api_version(PLUGIN_API_VERSION - 1).unwrap_err();

        assert!(matches!(
            error,
            LoaderError::ApiVersionMismatch {
                plugin_version,
                server_version,
            } if plugin_version == PLUGIN_API_VERSION - 1
                && server_version == PLUGIN_API_VERSION
        ));
    }

    #[test]
    fn accepts_current_native_plugin_metadata_layout() {
        assert!(
            validate_metadata_layout(size_of::<PluginMetadata>(), align_of::<PluginMetadata>())
                .is_ok()
        );
    }

    #[test]
    fn rejects_incompatible_native_plugin_metadata_layout() {
        let error = validate_metadata_layout(
            size_of::<PluginMetadata>() - size_of::<Vec<String>>(),
            align_of::<PluginMetadata>(),
        )
        .unwrap_err();

        assert!(matches!(error, LoaderError::MetadataLayoutMismatch { .. }));
    }
}
