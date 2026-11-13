import 'dart:io';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';

import 'ponte_det.dart';

void main() => runApp(const DetApp());
const cor = Color(0xFF264365);
const meses = [
  'Janeiro',
  'Fevereiro',
  'Marco',
  'Abril',
  'Maio',
  'Junho',
  'Julho',
  'Agosto',
  'Setembro',
  'Outubro',
  'Novembro',
  'Dezembro',
];

class DetApp extends StatelessWidget {
  const DetApp({super.key});
  @override
  Widget build(BuildContext context) => MaterialApp(
    title: 'Gestor de DETs',
    debugShowCheckedModeBanner: false,
    theme: ThemeData(
      useMaterial3: true,
      colorScheme: ColorScheme.fromSeed(seedColor: cor),
      scaffoldBackgroundColor: const Color(0xFFF3F6FA),
      cardTheme: CardThemeData(
        color: Colors.white,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(16),
          side: const BorderSide(color: Color(0xFFDCE4EE)),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        border: OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
      ),
    ),
    home: const Painel(),
  );
}

class Painel extends StatefulWidget {
  const Painel({super.key, this.carregarAoAbrir = true});
  final bool carregarAoAbrir;
  @override
  State<Painel> createState() => _PainelState();
}

class _PainelState extends State<Painel> {
  final api = const PonteDet();
  final base = TextEditingController(text: Directory.current.path);
  final card = TextEditingController();
  final planilha = TextEditingController();
  final modelo = TextEditingController();
  final limite = TextEditingController(text: '30');
  int fuso = -3;
  String? release;
  bool legendas = false, ocupado = false;
  String aviso = 'Escolha a pasta de trabalho.';
  Map<String, dynamic> dados = {};
  List<Map<String, dynamic>> get releases =>
      (dados['releases'] as List? ?? []).cast<Map<String, dynamic>>();
  bool get pronto =>
      release != null &&
      RegExp(r'^\d+$').hasMatch(card.text.trim()) &&
      releases.any(
        (r) =>
            r['nome'] == release &&
            (r['cards'] as List).contains('ID CARD ${card.text.trim()}'),
      );

  @override
  void initState() {
    super.initState();
    if (widget.carregarAoAbrir) {
      WidgetsBinding.instance.addPostFrameCallback((_) => atualizar());
    }
  }

  @override
  void dispose() {
    for (final c in [base, card, planilha, modelo, limite]) {
      c.dispose();
    }
    super.dispose();
  }

  Future<T?> trabalhar<T>(Future<T> Function() tarefa) async {
    if (ocupado) return null;
    setState(() => ocupado = true);
    try {
      return await tarefa();
    } catch (e) {
      if (mounted) {
        setState(() => aviso = e.toString().replaceFirst('Exception: ', ''));
      }
      return null;
    } finally {
      if (mounted) setState(() => ocupado = false);
    }
  }

  Future<void> atualizar() async => trabalhar(() async {
    final novo = await api.chamar('snapshot', base.text.trim(), fuso);
    if (!mounted) return;
    setState(() {
      dados = novo;
      if (!releases.any((r) => r['nome'] == release)) {
        release = null;
        card.clear();
      }
      if (planilha.text.isEmpty) {
        planilha.text = novo['planilha'] as String? ?? '';
      }
      if (modelo.text.isEmpty) modelo.text = novo['modelo'] as String? ?? '';
      aviso = 'Ambiente verificado. Selecione a Release e o ID CARD.';
    });
  });
  Future<void> escolherBase() async {
    final p = await getDirectoryPath(confirmButtonText: 'Usar esta pasta');
    if (p == null) return;
    setState(() {
      base.text = p;
      release = null;
      card.clear();
      planilha.clear();
      modelo.clear();
    });
    await atualizar();
  }

  Future<void> escolherArquivo(TextEditingController c, String ext) async {
    final p = await openFile(
      acceptedTypeGroups: [
        XTypeGroup(label: ext.toUpperCase(), extensions: [ext]),
      ],
    );
    if (p != null) setState(() => c.text = p.path);
  }

  Future<void> novaRelease() async {
    int mes = DateTime.now().month, ano = DateTime.now().year;
    final nome = await showDialog<String>(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (ctx, mudar) => AlertDialog(
          title: const Text('Nova Release'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              DropdownButtonFormField<int>(
                initialValue: mes,
                decoration: const InputDecoration(labelText: 'Mês'),
                items: List.generate(
                  12,
                  (i) => DropdownMenuItem(value: i + 1, child: Text(meses[i])),
                ),
                onChanged: (v) => mudar(() => mes = v ?? mes),
              ),
              const SizedBox(height: 12),
              TextFormField(
                initialValue: '$ano',
                keyboardType: TextInputType.number,
                decoration: const InputDecoration(labelText: 'Ano'),
                onChanged: (v) => ano = int.tryParse(v) ?? 0,
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(ctx),
              child: const Text('Cancelar'),
            ),
            FilledButton(
              onPressed: () {
                if (ano >= 2020 && ano <= 2100) {
                  Navigator.pop(ctx, 'Release ${meses[mes - 1]} $ano');
                }
              },
              child: const Text('Usar Release'),
            ),
          ],
        ),
      ),
    );
    if (nome != null && mounted) setState(() => release = nome);
  }

  Future<void> selecionar() async => trabalhar(() async {
    if (release == null) throw Exception('Escolha uma Release.');
    if (!RegExp(r'^\d+$').hasMatch(card.text.trim())) {
      throw Exception('O ID CARD deve ser numérico.');
    }
    final resultado = await api.chamar('contexto', base.text.trim(), fuso, [
      release!,
      card.text.trim(),
    ]);
    final novo = await api.chamar('snapshot', base.text.trim(), fuso);
    if (mounted) {
      setState(() {
        dados = novo;
        aviso = resultado['mensagem'] as String;
      });
    }
  });
  Future<void> executar(String acao, [List<String> extras = const []]) async {
    if (!pronto) {
      setState(() => aviso = 'Selecione uma Release e um ID CARD primeiro.');
      return;
    }
    await trabalhar(() async {
      final args = [release!, card.text.trim(), ...extras];
      if (acao == 'docx') {
        final resultado = await api.chamar(
          'ambiguidade',
          base.text.trim(),
          fuso,
          [release!, card.text.trim(), planilha.text.trim()],
        );
        for (final item
            in (resultado['itens'] as List? ?? [])
                .cast<Map<String, dynamic>>()) {
          if (!mounted) return;
          final escolha = await showDialog<String>(
            context: context,
            barrierDismissible: false,
            builder: (ctx) => SimpleDialog(
              title: Text('Escolha a pasta do ID ${item['id']}'),
              children: [
                for (final pasta in (item['pastas'] as List).cast<String>())
                  SimpleDialogOption(
                    onPressed: () => Navigator.pop(ctx, pasta),
                    child: Text(pasta),
                  ),
                SimpleDialogOption(
                  onPressed: () => Navigator.pop(ctx),
                  child: const Text('Cancelar'),
                ),
              ],
            ),
          );
          if (escolha == null) {
            setState(() => aviso = 'Geração cancelada.');
            return;
          }
          args.add('${item['id']}=$escolha');
        }
      }
      final resultado = await api.chamar(acao, base.text.trim(), fuso, args);
      if (mounted) {
        setState(
          () => aviso = resultado['mensagem'] as String? ?? 'Concluído.',
        );
      }
    });
  }

  Widget secao(String titulo, Widget filho) => Card(
    child: Padding(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            titulo,
            style: const TextStyle(
              color: cor,
              fontSize: 19,
              fontWeight: FontWeight.bold,
            ),
          ),
          const SizedBox(height: 16),
          filho,
        ],
      ),
    ),
  );
  Widget arquivo(String titulo, TextEditingController c, String ext) => Row(
    children: [
      Expanded(
        child: TextField(
          controller: c,
          decoration: InputDecoration(labelText: titulo),
        ),
      ),
      const SizedBox(width: 8),
      IconButton.filledTonal(
        onPressed: ocupado ? null : () => escolherArquivo(c, ext),
        tooltip: 'Escolher $titulo',
        icon: const Icon(Icons.folder_open),
      ),
    ],
  );
  Widget passo(
    int n,
    String titulo,
    String detalhe,
    IconData icone,
    VoidCallback? rodar,
  ) => Padding(
    padding: const EdgeInsets.only(bottom: 10),
    child: Row(
      children: [
        CircleAvatar(
          backgroundColor: const Color(0xFFE6EDF7),
          child: Text(
            '$n',
            style: const TextStyle(color: cor, fontWeight: FontWeight.bold),
          ),
        ),
        const SizedBox(width: 12),
        Icon(icone, color: cor),
        const SizedBox(width: 12),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(titulo, style: const TextStyle(fontWeight: FontWeight.w700)),
              Text(detalhe),
            ],
          ),
        ),
        const SizedBox(width: 8),
        FilledButton(
          onPressed: ocupado ? null : rodar,
          child: const Text('Executar'),
        ),
      ],
    ),
  );

  @override
  Widget build(BuildContext context) {
    final nomesReleases = releases.map((r) => r['nome'] as String).toList();
    if (release != null && !nomesReleases.contains(release)) {
      nomesReleases.add(release!);
    }
    final esquerda = Column(
      children: [
        secao(
          'Pasta de trabalho',
          Column(
            children: [
              Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: base,
                      decoration: const InputDecoration(labelText: 'Caminho'),
                      onSubmitted: (_) => atualizar(),
                    ),
                  ),
                  const SizedBox(width: 8),
                  IconButton.filledTonal(
                    onPressed: ocupado ? null : escolherBase,
                    tooltip: 'Escolher pasta',
                    icon: const Icon(Icons.folder_open),
                  ),
                ],
              ),
              Align(
                alignment: Alignment.centerRight,
                child: TextButton.icon(
                  onPressed: ocupado ? null : atualizar,
                  icon: const Icon(Icons.refresh),
                  label: const Text('Atualizar'),
                ),
              ),
            ],
          ),
        ),
        secao(
          'Release e card',
          Column(
            children: [
              DropdownButtonFormField<String>(
                key: ValueKey(release),
                initialValue: release,
                decoration: const InputDecoration(labelText: 'Release'),
                items: nomesReleases
                    .map(
                      (nome) => DropdownMenuItem(
                        value: nome,
                        child: Text(nome),
                      ),
                    )
                    .toList(),
                onChanged: (v) => setState(() {
                  release = v;
                  card.clear();
                }),
              ),
              Align(
                alignment: Alignment.centerLeft,
                child: TextButton.icon(
                  onPressed: ocupado ? null : novaRelease,
                  icon: const Icon(Icons.add),
                  label: const Text('Nova Release'),
                ),
              ),
              TextField(
                controller: card,
                keyboardType: TextInputType.number,
                decoration: const InputDecoration(
                  labelText: 'Número do ID CARD',
                ),
                onChanged: (_) => setState(() {}),
              ),
              const SizedBox(height: 12),
              SizedBox(
                width: double.infinity,
                child: FilledButton(
                  onPressed: ocupado ? null : selecionar,
                  child: const Text('Selecionar ou criar card'),
                ),
              ),
            ],
          ),
        ),
        secao(
          'Arquivos de entrada',
          Column(
            children: [
              arquivo('Planilha de testes', planilha, 'xlsx'),
              const SizedBox(height: 12),
              arquivo('Modelo do DET', modelo, 'docx'),
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                title: const Text('Legendas sob as imagens'),
                value: legendas,
                onChanged: (v) => setState(() => legendas = v),
              ),
            ],
          ),
        ),
      ],
    );
    final direita = Column(
      children: [
        secao(
          'Fluxo de trabalho',
          Column(
            children: [
              passo(
                1,
                'Criar subpastas',
                'Uma por teste da planilha.',
                Icons.create_new_folder_outlined,
                pronto
                    ? () => executar('organizar', [planilha.text.trim()])
                    : null,
              ),
              passo(
                2,
                'Descompactar evidências',
                'Extrai imagens dos ZIPs.',
                Icons.folder_zip_outlined,
                pronto ? () => executar('descompactar') : null,
              ),
              passo(
                3,
                'Gerar DETs em Word',
                'Preenche o modelo e inclui imagens.',
                Icons.description_outlined,
                pronto
                    ? () => executar('docx', [
                        planilha.text.trim(),
                        modelo.text.trim(),
                        '$legendas',
                      ])
                    : null,
              ),
              passo(
                4,
                'Converter para PDF',
                'Requer LibreOffice.',
                Icons.picture_as_pdf_outlined,
                pronto ? () => executar('pdf') : null,
              ),
              TextField(
                controller: limite,
                keyboardType: TextInputType.number,
                decoration: const InputDecoration(
                  labelText: 'Limite por compilado (MB)',
                ),
              ),
              const SizedBox(height: 10),
              passo(
                5,
                'Gerar compilado',
                'Junta os PDFs; requer qpdf.',
                Icons.library_books_outlined,
                pronto
                    ? () => executar('compilado', [limite.text.trim()])
                    : null,
              ),
            ],
          ),
        ),
        secao(
          'Ambiente',
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              for (final (nome, chave) in [
                ('LibreOffice', 'libreoffice'),
                ('qpdf', 'qpdf'),
                ('Ghostscript', 'ghostscript'),
              ])
                Chip(
                  avatar: Icon(
                    dados[chave] == true
                        ? Icons.check_circle
                        : Icons.error_outline,
                    size: 18,
                    color: dados[chave] == true ? Colors.green : Colors.orange,
                  ),
                  label: Text(nome),
                ),
              Chip(
                label: Text(
                  'Data: ${dados['data'] ?? '—'} (UTC${fuso < 0 ? '' : '+'}$fuso)',
                ),
              ),
            ],
          ),
        ),
      ],
    );
    final estreita = MediaQuery.sizeOf(context).width < 850;
    return Scaffold(
      appBar: AppBar(
        backgroundColor: cor,
        foregroundColor: Colors.white,
        title: const Text('Gestor de Documentos de Testes'),
        actions: [
          const Center(child: Text('Fuso UTC')),
          const SizedBox(width: 8),
          DropdownButton<int>(
            value: fuso,
            dropdownColor: cor,
            style: const TextStyle(color: Colors.white),
            items: List.generate(27, (i) => i - 12)
                .map(
                  (n) => DropdownMenuItem(
                    value: n,
                    child: Text('${n >= 0 ? '+' : ''}$n'),
                  ),
                )
                .toList(),
            onChanged: ocupado
                ? null
                : (v) {
                    setState(() => fuso = v ?? -3);
                    atualizar();
                  },
          ),
          const SizedBox(width: 20),
        ],
      ),
      body: Column(
        children: [
          if (ocupado) const LinearProgressIndicator(),
          Expanded(
            child: SingleChildScrollView(
              child: Center(
                child: ConstrainedBox(
                  constraints: const BoxConstraints(maxWidth: 1380),
                  child: Padding(
                    padding: const EdgeInsets.all(18),
                    child: estreita
                        ? Column(children: [esquerda, direita])
                        : Row(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Expanded(flex: 2, child: esquerda),
                              const SizedBox(width: 16),
                              Expanded(flex: 3, child: direita),
                            ],
                          ),
                  ),
                ),
              ),
            ),
          ),
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(14),
            color: const Color(0xFFE1EAF5),
            child: Text(aviso, style: const TextStyle(color: cor)),
          ),
        ],
      ),
    );
  }
}
