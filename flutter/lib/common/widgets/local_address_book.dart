import 'package:flutter/material.dart';

import '../../common.dart';
import '../../models/local_address_book_model.dart';

class LocalAddressBook extends StatefulWidget {
  const LocalAddressBook({Key? key}) : super(key: key);

  @override
  State<LocalAddressBook> createState() => _LocalAddressBookState();
}

class _LocalAddressBookState extends State<LocalAddressBook> {
  final LocalAddressBookModel model = LocalAddressBookModel();
  final TextEditingController search = TextEditingController();

  @override
  void initState() {
    super.initState();
    model.addListener(_refresh);
    model.load();
  }

  @override
  void dispose() {
    model.removeListener(_refresh);
    search.dispose();
    super.dispose();
  }

  void _refresh() {
    if (mounted) setState(() {});
  }

  List<LocalAddressBookEntry> get filteredEntries {
    final q = search.text.trim().toLowerCase();
    if (q.isEmpty) return model.entries;
    return model.entries.where((e) {
      return e.id.toLowerCase().contains(q) ||
          e.alias.toLowerCase().contains(q) ||
          e.group.toLowerCase().contains(q) ||
          e.note.toLowerCase().contains(q);
    }).toList();
  }

  Future<String?> _textDialog(String title, String initial) async {
    final controller = TextEditingController(text: initial);
    final result = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(title),
        content: TextField(
          controller: controller,
          autofocus: true,
          onSubmitted: (v) => Navigator.of(context).pop(v.trim()),
        ),
        actions: [
          TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Отмена')),
          FilledButton(
              onPressed: () =>
                  Navigator.of(context).pop(controller.text.trim()),
              child: const Text('Сохранить')),
        ],
      ),
    );
    controller.dispose();
    return result;
  }

  Future<void> _addGroup() async {
    final name = await _textDialog('Новая группа', '');
    if (name != null && name.isNotEmpty) {
      await model.addGroup(name);
    }
  }

  Future<void> _editGroup(String group) async {
    final name = await _textDialog('Переименовать группу', group);
    if (name != null && name.isNotEmpty) {
      await model.renameGroup(group, name);
    }
  }

  Future<void> _deleteGroup(String group) async {
    final ok = await showDialog<bool>(
          context: context,
          builder: (context) => AlertDialog(
            title: const Text('Удалить группу?'),
            content: Text(
                'Устройства из группы «$group» останутся в адресной книге без группы.'),
            actions: [
              TextButton(
                  onPressed: () => Navigator.of(context).pop(false),
                  child: const Text('Отмена')),
              FilledButton(
                  onPressed: () => Navigator.of(context).pop(true),
                  child: const Text('Удалить')),
            ],
          ),
        ) ??
        false;
    if (ok) await model.deleteGroup(group);
  }

  Future<void> _editEntry([LocalAddressBookEntry? current]) async {
    final id = TextEditingController(text: current?.id ?? '');
    final alias = TextEditingController(text: current?.alias ?? '');
    final note = TextEditingController(text: current?.note ?? '');
    String group = current?.group ?? '';
    final result = await showDialog<LocalAddressBookEntry>(
      context: context,
      builder: (context) => StatefulBuilder(builder: (context, setDialogState) {
        return AlertDialog(
          title: Text(current == null ? 'Добавить устройство' : 'Редактировать устройство'),
          content: SizedBox(
            width: 430,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextField(
                  controller: alias,
                  decoration: const InputDecoration(labelText: 'Название'),
                  autofocus: true,
                ),
                const SizedBox(height: 8),
                TextField(
                  controller: id,
                  decoration: const InputDecoration(labelText: 'RustDesk ID / IP'),
                ),
                const SizedBox(height: 8),
                DropdownButtonFormField<String>(
                  value: model.groups.contains(group) ? group : '',
                  decoration: const InputDecoration(labelText: 'Группа'),
                  items: [
                    const DropdownMenuItem(value: '', child: Text('Без группы')),
                    ...model.groups.map(
                        (g) => DropdownMenuItem(value: g, child: Text(g))),
                  ],
                  onChanged: (v) => setDialogState(() => group = v ?? ''),
                ),
                const SizedBox(height: 8),
                TextField(
                  controller: note,
                  maxLines: 3,
                  decoration: const InputDecoration(labelText: 'Заметка'),
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
                onPressed: () => Navigator.of(context).pop(),
                child: const Text('Отмена')),
            FilledButton(
              onPressed: () {
                final value = id.text.replaceAll(' ', '').trim();
                if (value.isEmpty) return;
                Navigator.of(context).pop(LocalAddressBookEntry(
                  id: value,
                  alias: alias.text.trim(),
                  group: group,
                  note: note.text.trim(),
                ));
              },
              child: const Text('Сохранить'),
            ),
          ],
        );
      }),
    );
    id.dispose();
    alias.dispose();
    note.dispose();
    if (result != null) {
      await model.upsertEntry(result, originalId: current?.id);
    }
  }

  Future<void> _deleteEntry(LocalAddressBookEntry entry) async {
    final ok = await showDialog<bool>(
          context: context,
          builder: (context) => AlertDialog(
            title: const Text('Удалить устройство?'),
            content: Text(entry.alias.isEmpty ? entry.id : entry.alias),
            actions: [
              TextButton(
                  onPressed: () => Navigator.of(context).pop(false),
                  child: const Text('Отмена')),
              FilledButton(
                  onPressed: () => Navigator.of(context).pop(true),
                  child: const Text('Удалить')),
            ],
          ),
        ) ??
        false;
    if (ok) await model.deleteEntry(entry.id);
  }

  Widget _entryTile(LocalAddressBookEntry entry) {
    return GestureDetector(
      onDoubleTap: () => connect(context, entry.id),
      child: Card(
        margin: const EdgeInsets.symmetric(vertical: 3, horizontal: 2),
        child: ListTile(
          dense: true,
          leading: const Icon(Icons.computer),
          title: Text(entry.alias.isEmpty ? entry.id : entry.alias),
          subtitle: Text([
            entry.id,
            if (entry.note.isNotEmpty) entry.note,
          ].join('  •  ')),
          trailing: PopupMenuButton<String>(
            onSelected: (value) {
              if (value == 'connect') connect(context, entry.id);
              if (value == 'edit') _editEntry(entry);
              if (value == 'delete') _deleteEntry(entry);
            },
            itemBuilder: (_) => const [
              PopupMenuItem(value: 'connect', child: Text('Подключиться')),
              PopupMenuItem(value: 'edit', child: Text('Редактировать')),
              PopupMenuItem(value: 'delete', child: Text('Удалить')),
            ],
          ),
        ),
      ),
    );
  }

  Widget _groupBlock(String group, List<LocalAddressBookEntry> items) {
    return ExpansionTile(
      initiallyExpanded: true,
      leading: const Icon(Icons.folder_outlined),
      title: Text(group),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          IconButton(
              tooltip: 'Переименовать',
              onPressed: () => _editGroup(group),
              icon: const Icon(Icons.edit_outlined, size: 18)),
          IconButton(
              tooltip: 'Удалить группу',
              onPressed: () => _deleteGroup(group),
              icon: const Icon(Icons.delete_outline, size: 18)),
        ],
      ),
      children: items.map(_entryTile).toList(),
    );
  }

  @override
  Widget build(BuildContext context) {
    if (!model.loaded) {
      return const Center(child: CircularProgressIndicator());
    }
    final entries = filteredEntries;
    final noGroup = entries.where((e) => e.group.isEmpty).toList();

    return Column(
      children: [
        Row(
          children: [
            Expanded(
              child: TextField(
                controller: search,
                onChanged: (_) => setState(() {}),
                decoration: const InputDecoration(
                  isDense: true,
                  prefixIcon: Icon(Icons.search),
                  hintText: 'Поиск по адресной книге',
                ),
              ),
            ),
            const SizedBox(width: 8),
            OutlinedButton.icon(
              onPressed: _addGroup,
              icon: const Icon(Icons.create_new_folder_outlined),
              label: const Text('Группа'),
            ),
            const SizedBox(width: 8),
            FilledButton.icon(
              onPressed: () => _editEntry(),
              icon: const Icon(Icons.add),
              label: const Text('Устройство'),
            ),
          ],
        ),
        const SizedBox(height: 8),
        Expanded(
          child: entries.isEmpty
              ? const Center(
                  child: Text(
                    'Адресная книга пуста.\nДобавьте группу или устройство.',
                    textAlign: TextAlign.center,
                  ),
                )
              : ListView(
                  children: [
                    for (final group in model.groups)
                      if (entries.any((e) => e.group == group))
                        _groupBlock(
                            group, entries.where((e) => e.group == group).toList()),
                    if (noGroup.isNotEmpty)
                      _groupBlock('Без группы', noGroup),
                  ],
                ),
        ),
      ],
    );
  }
}
