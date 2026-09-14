//! The ordered list of files to play and the current selection.

use std::{
	io,
	path::{Path, PathBuf},
};

const MIDI_EXTENSIONS: [&str; 2] = ["mid", "midi"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Track {
	pub path: PathBuf,
	/// The file stem, shown until the file itself supplies a title.
	pub title: String,
}

impl Track {
	fn new(path: PathBuf) -> Self {
		let title = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
		Self { path, title }
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Playlist {
	tracks: Vec<Track>,
	current: Option<usize>,
}

impl Playlist {
	#[must_use]
	pub fn tracks(&self) -> &[Track] {
		&self.tracks
	}

	#[must_use]
	pub const fn current(&self) -> Option<usize> {
		self.current
	}

	#[must_use]
	pub fn current_track(&self) -> Option<&Track> {
		self.current.and_then(|i| self.tracks.get(i))
	}

	/// Appends files in the given order; an empty playlist selects the first one.
	pub fn add_files(&mut self, paths: impl IntoIterator<Item = PathBuf>) {
		self.tracks.extend(paths.into_iter().map(Track::new));
		if self.current.is_none() && !self.tracks.is_empty() {
			self.current = Some(0);
		}
	}

	/// Appends the MIDI files directly inside `dir`, sorted by name, and returns how many.
	pub fn add_folder(&mut self, dir: &Path) -> io::Result<usize> {
		let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)?
			.filter_map(Result::ok)
			.map(|entry| entry.path())
			.filter(|path| path.is_file() && is_midi_file(path))
			.collect();
		paths.sort_by_cached_key(|path| path.file_name().map(|n| n.to_string_lossy().to_lowercase()));
		let added = paths.len();
		self.add_files(paths);
		Ok(added)
	}

	/// Removes the track at `index`, keeping the current selection on the same track where
	/// possible and otherwise moving it to the previous one.
	pub fn remove(&mut self, index: usize) {
		if index >= self.tracks.len() {
			return;
		}
		self.tracks.remove(index);
		self.current = match self.current {
			Some(current) if current > index => Some(current - 1),
			Some(current) if current < self.tracks.len() => Some(current),
			Some(_) => self.tracks.len().checked_sub(1),
			None => None,
		};
	}

	pub fn clear(&mut self) {
		self.tracks.clear();
		self.current = None;
	}

	/// Makes `index` current; returns whether it exists.
	pub const fn select(&mut self, index: usize) -> bool {
		if index < self.tracks.len() {
			self.current = Some(index);
			true
		} else {
			false
		}
	}

	/// Advances to the following track, or returns `None` at the end.
	pub fn select_next(&mut self) -> Option<usize> {
		let next = self.current? + 1;
		self.select(next).then_some(next)
	}

	/// Goes back to the preceding track, or returns `None` at the start.
	pub fn select_previous(&mut self) -> Option<usize> {
		let previous = self.current?.checked_sub(1)?;
		self.select(previous).then_some(previous)
	}
}

fn is_midi_file(path: &Path) -> bool {
	path.extension().and_then(|e| e.to_str()).is_some_and(|e| MIDI_EXTENSIONS.iter().any(|m| m.eq_ignore_ascii_case(e)))
}

#[cfg(test)]
mod tests {
	use std::{fs, path::PathBuf};

	use super::*;

	fn paths(names: &[&str]) -> Vec<PathBuf> {
		names.iter().map(PathBuf::from).collect()
	}

	fn titles(playlist: &Playlist) -> Vec<&str> {
		playlist.tracks().iter().map(|t| t.title.as_str()).collect()
	}

	#[test]
	fn added_files_use_their_file_stem_as_title() {
		let mut playlist = Playlist::default();
		playlist.add_files(paths(&["C:/music/First Song.mid", "C:/music/second.MIDI"]));
		assert_eq!(titles(&playlist), ["First Song", "second"]);
	}

	#[test]
	fn adding_files_to_an_empty_playlist_selects_the_first() {
		let mut playlist = Playlist::default();
		playlist.add_files(paths(&["a.mid", "b.mid"]));
		assert_eq!(playlist.current(), Some(0));
		playlist.add_files(paths(&["c.mid"]));
		assert_eq!(playlist.current(), Some(0));
	}

	#[test]
	fn add_folder_takes_midi_files_sorted_by_name_ignoring_case() {
		let dir = tempfile::tempdir().unwrap();
		for name in ["zeta.mid", "Alpha.MID", "notes.txt", "beta.midi"] {
			fs::write(dir.path().join(name), b"").unwrap();
		}
		fs::create_dir(dir.path().join("sub.mid")).unwrap();
		let mut playlist = Playlist::default();
		let added = playlist.add_folder(dir.path()).unwrap();
		assert_eq!(added, 3);
		assert_eq!(titles(&playlist), ["Alpha", "beta", "zeta"]);
	}

	#[test]
	fn next_and_previous_stop_at_the_ends() {
		let mut playlist = Playlist::default();
		playlist.add_files(paths(&["a.mid", "b.mid"]));
		assert_eq!(playlist.select_previous(), None);
		assert_eq!(playlist.select_next(), Some(1));
		assert_eq!(playlist.select_next(), None);
		assert_eq!(playlist.current(), Some(1));
		assert_eq!(playlist.select_previous(), Some(0));
	}

	#[test]
	fn select_rejects_out_of_range_indexes() {
		let mut playlist = Playlist::default();
		playlist.add_files(paths(&["a.mid"]));
		assert!(!playlist.select(1));
		assert_eq!(playlist.current(), Some(0));
		assert!(playlist.select(0));
	}

	#[test]
	fn removing_before_the_current_track_keeps_it_current() {
		let mut playlist = Playlist::default();
		playlist.add_files(paths(&["a.mid", "b.mid", "c.mid"]));
		playlist.select(2);
		playlist.remove(0);
		assert_eq!(playlist.current(), Some(1));
		assert_eq!(playlist.current_track().unwrap().title, "c");
	}

	#[test]
	fn removing_the_last_current_track_moves_selection_back() {
		let mut playlist = Playlist::default();
		playlist.add_files(paths(&["a.mid", "b.mid"]));
		playlist.select(1);
		playlist.remove(1);
		assert_eq!(playlist.current(), Some(0));
		playlist.remove(0);
		assert_eq!(playlist.current(), None);
	}

	#[test]
	fn clear_empties_everything() {
		let mut playlist = Playlist::default();
		playlist.add_files(paths(&["a.mid"]));
		playlist.clear();
		assert!(playlist.tracks().is_empty());
		assert_eq!(playlist.current(), None);
	}
}
