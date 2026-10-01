import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter_hbb/models/platform_model.dart';
import '../common.dart';

const _localAddressBookKey = 'oameit-local-address-book';

class LocalAddressBookEntry {
  String id;
  String alias;
  String group;
  String note;

  LocalAddressBookEntry({
    required this.id,
    required this.alias,
    required this.group,
    required this.note,
  });

  factory LocalAddressBookEntry.fromJson(Map<String, dynamic> json) {
    return LocalAddressBookEntry(
      id: (json['id'] ?? '').toString(),
      alias: (json['alias'] ?? '').toString(),
      group: (json['group'] ?? '').toString(),
      note: (json['note'] ?? '').toString(),
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'alias': alias,
        'group': group,
        'note': note,
      };
}

class LocalAddressBookModel extends ChangeNotifier {
  final List<String> groups = [];
  final List<LocalAddressBookEntry> entries = [];
  bool loaded = false;

  Future<void> load() async {
    if (loaded) return;
    try {
      final raw = bind.mainGetLocalOption(key: _localAddressBookKey);
      if (raw.isNotEmpty) {
        final decoded = jsonDecode(raw);
        if (decoded is Map<String, dynamic>) {
          final decodedGroups = decoded['groups'];
          if (decodedGroups is List) {
            groups
              ..clear()
              ..addAll(decodedGroups.map((e) => e.toString()).where((e) => e.isNotEmpty));
          }
          final decodedEntries = decoded['entries'];
          if (decodedEntries is List) {
            entries
              ..clear()
              ..addAll(decodedEntries
                  .whereType<Map>()
                  .map((e) => LocalAddressBookEntry.fromJson(
                      Map<String, dynamic>.from(e))));
          }
        }
      }
    } catch (e) {
      debugPrint('Failed to load local address book: $e');
    }
    loaded = true;
    notifyListeners();
  }

  Future<void> _save() async {
    final payload = jsonEncode({
      'groups': groups,
      'entries': entries.map((e) => e.toJson()).toList(),
    });
    await bind.mainSetLocalOption(key: _localAddressBookKey, value: payload);
    notifyListeners();
  }

  Future<void> addGroup(String name) async {
    final value = name.trim();
    if (value.isEmpty || groups.contains(value)) return;
    groups.add(value);
    groups.sort((a, b) => a.toLowerCase().compareTo(b.toLowerCase()));
    await _save();
  }

  Future<void> renameGroup(String oldName, String newName) async {
    final value = newName.trim();
    if (value.isEmpty || oldName == value) return;
    final idx = groups.indexOf(oldName);
    if (idx < 0) return;
    groups[idx] = value;
    for (final entry in entries) {
      if (entry.group == oldName) entry.group = value;
    }
    groups.sort((a, b) => a.toLowerCase().compareTo(b.toLowerCase()));
    await _save();
  }

  Future<void> deleteGroup(String name) async {
    groups.remove(name);
    for (final entry in entries) {
      if (entry.group == name) entry.group = '';
    }
    await _save();
  }

  Future<void> upsertEntry(LocalAddressBookEntry entry,
      {String? originalId}) async {
    final normalizedId = entry.id.replaceAll(' ', '').trim();
    if (normalizedId.isEmpty) return;
    entry.id = normalizedId;
    final targetId = originalId ?? normalizedId;
    final idx = entries.indexWhere((e) => e.id == targetId);
    if (idx >= 0) {
      entries[idx] = entry;
    } else {
      entries.add(entry);
    }
    entries.sort((a, b) =>
        (a.alias.isEmpty ? a.id : a.alias).toLowerCase().compareTo(
            (b.alias.isEmpty ? b.id : b.alias).toLowerCase()));
    await _save();
  }

  Future<void> deleteEntry(String id) async {
    entries.removeWhere((e) => e.id == id);
    await _save();
  }
}
