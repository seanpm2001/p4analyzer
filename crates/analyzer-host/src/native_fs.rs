// Web Assembly is a sandbox, and by design shouldn't have access to network, file systenm, or underlying operating system.
// Simply removing this file system code from wasm build is the best way
#[cfg(not(target_arch = "wasm32"))]
pub mod native_fs {
	use std::{path::PathBuf, sync::Arc, thread, any::Any};

	use analyzer_abstractions::{
		fs::EnumerableFileSystem,
		lsp_types::{FileChangeType, FileEvent, TextDocumentIdentifier, Url},
		BoxFuture,
	};
	use async_channel::Sender;
	use cancellation::CancellationToken;
	use futures::lock::Mutex;
	use notify::{Event, RecursiveMode, Watcher};
	use regex::Regex;

	use crate::json_rpc::message::{Message, Notification};

	pub struct NativeFs {
		watcher: notify::RecommendedWatcher,
		watching: Vec<Url>,
	}

	impl NativeFs {
		pub fn new(token: Arc<CancellationToken>, request_sender: Sender<Message>) -> Arc<Mutex<Box<dyn EnumerableFileSystem + Send + Sync + 'static>>> {
			let watcher = notify::recommended_watcher(move |res| match res {
				Ok(event) => {
					if let Some(mess) = Self::file_change(event) {
						let _ = futures::executor::block_on(request_sender.send(mess));
					}
				}
				Err(e) => println!("watch error: {:?}", e),
			})
			.unwrap();

			let object: Arc<Mutex<Box<dyn EnumerableFileSystem + Send + Sync + 'static>>> = Arc::new(Mutex::new(Box::new(NativeFs { watcher, watching: Vec::new() })));

			// Start a new thread as CancellationToken::run() is blocking
			// only 1 FileSystem should exist per P4Analyzer so not to worried about many child threads
			let clone = object.clone();
			thread::spawn(move || {
				token.run(|| {
					futures::executor::block_on(clone.lock()).as_any().downcast_mut::<NativeFs>().unwrap().stop_watching_all();
				}, 
				|| {});
			});

			object
		}

		// No current way to called the `watcher.unwatch()` function
		fn start_folder_watch(&mut self, folder_uri: &Url) {
			self.watching.push(folder_uri.clone()); // add path to vector
			self.watcher.watch(folder_uri.path().as_ref(), RecursiveMode::Recursive).unwrap(); // start watcher		
		}

		fn stop_watching_all(&mut self) {
			for elm in &self.watching {
				self.watcher.unwatch(elm.path().as_ref()).unwrap();
			}
			self.watching.clear();
		}

		// has to be manually called
		pub fn stop_folder_watch(&mut self, folder_uri: &Url) {
			self.watching.retain(|x| *x != *folder_uri); // remove from vector
			self.watcher.unwatch(folder_uri.path().as_ref()).unwrap();	// if exists with unwatch it
		}

		fn file_change(event: Event) -> Option<Message> {
			let paths = event.paths;
			match event.kind {
				notify::EventKind::Any => None,
				notify::EventKind::Access(_) => None,
				notify::EventKind::Create(_) => Self::create_message(paths, FileChangeType::CREATED),
				notify::EventKind::Modify(_) => Self::create_message(paths, FileChangeType::CHANGED),
				notify::EventKind::Remove(_) => Self::create_message(paths, FileChangeType::DELETED),
				notify::EventKind::Other => None,
			}
		}

		fn create_message(paths: Vec<PathBuf>, event_type: FileChangeType) -> Option<Message> {
			let files = paths
				.into_iter()
				.map(|x| FileEvent { uri: Url::parse(x.to_str().unwrap()).unwrap(), typ: event_type })
				.collect();

			let create_files_params = analyzer_abstractions::lsp_types::DidChangeWatchedFilesParams { changes: files };
			let params = serde_json::json!(create_files_params);

			// no sure of the difference between `workspace/didChangeWatchedFiles` and `workspace/didDeleteFiles` or `workspace/didCreateFiles`
			Some(Message::Notification(Notification { method: "workspace/didChangeWatchedFiles".into(), params }))
		}
	}

	// EnumerableFileSystem part of NativeFs will just use std::fs methods for the functions
	impl EnumerableFileSystem for NativeFs {
		fn as_any(&mut self) -> &mut dyn Any {
			self
		}

		fn enumerate_folder<'a>(
			&'a mut self,
			folder_uri: Url,
			file_pattern: String,
		) -> BoxFuture<'a, Vec<TextDocumentIdentifier>> {
			self.start_folder_watch(&folder_uri); // add folder to watch list

			async fn enumerate_folder(folder_uri: Url, file_pattern: String) -> Vec<TextDocumentIdentifier> {
				let folder = folder_uri.path();

				let res = std::fs::read_dir(folder); // make async somehow
				if res.is_err() {
					return Vec::new();
				}
				let dir_itr = res.unwrap();

				let re = Regex::new(file_pattern.as_str()).unwrap();
				let mut output = Vec::new();

				for file in dir_itr {
					// make async somehow
					if re.is_match(file.as_ref().unwrap().file_name().to_str().unwrap()) {
						let path = file.unwrap().path();
						output.push(TextDocumentIdentifier { uri: Url::parse(path.to_str().unwrap()).unwrap() })
					}
				}

				output
			}

			Box::pin(enumerate_folder(folder_uri, file_pattern))
		}

		fn file_contents<'a>(&'a self, file_uri: Url) -> BoxFuture<'a, Option<String>> {
			async fn file_contents(file_uri: Url) -> Option<String> {
				let path = file_uri.path();

				let data = tokio::fs::read(path).await;

				match data {
					Ok(data) => Some(String::from_utf8(data).unwrap()),
					Err(_) => None,
				}
			}

			Box::pin(file_contents(file_uri))
		}
	}
}

// add wasm32 version for build to be completed but it's unreachable code
#[cfg(target_arch = "wasm32")]
pub mod native_fs {
	use std::sync::Arc;
	use std::any::Any;
	use analyzer_abstractions::fs::EnumerableFileSystem;
	use async_channel::Sender;
	use cancellation::CancellationToken;
use futures::lock::Mutex;

	use crate::json_rpc::message::Message;

	pub struct NativeFs {}

	impl NativeFs {
		pub fn new(_: Arc<CancellationToken>, _: Sender<Message>) -> Arc<Mutex<Box<dyn EnumerableFileSystem + Send + Sync + 'static>>> {
			unreachable!("Wasm run-time reached native only code: NativeFs::new() !!!")
		}
	}

	impl EnumerableFileSystem for NativeFs {
		fn as_any(&mut self) -> &mut dyn Any {
			unreachable!("Wasm run-time reached native only code: NativeFs::as_any() !!!")
		}

		fn enumerate_folder<'a>(
			&'a mut self,
			_: analyzer_abstractions::lsp_types::Url,
			_: String,
		) -> analyzer_abstractions::BoxFuture<'a, Vec<analyzer_abstractions::lsp_types::TextDocumentIdentifier>> {
			unreachable!("Wasm run-time reached native only code: NativeFs::enumerate_folder() !!!")
		}

		fn file_contents<'a>(
			&'a self,
			_: analyzer_abstractions::lsp_types::Url,
		) -> analyzer_abstractions::BoxFuture<'a, Option<String>> {
			unreachable!("Wasm run-time reached native only code: NativeFs::file_contents() !!!")
		}
	}
}
