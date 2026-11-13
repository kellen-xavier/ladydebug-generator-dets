import 'dart:convert';
import 'dart:io';

class PonteDet {
  const PonteDet();

  String get executavel {
    final nome = Platform.isWindows ? 'det-api.exe' : 'det-api';
    final definido = Platform.environment['DET_API_BIN'];
    if (definido != null && definido.isNotEmpty) return definido;
    final vizinho = '${File(Platform.resolvedExecutable).parent.path}/$nome';
    if (File(vizinho).existsSync()) return vizinho;
    for (final p in [
      'target/debug/$nome',
      '../target/debug/$nome',
      'target/release/$nome',
      '../target/release/$nome',
    ]) {
      if (File(p).existsSync()) return File(p).absolute.path;
    }
    return vizinho;
  }

  Future<Map<String, dynamic>> chamar(
    String acao,
    String base,
    int fuso, [
    List<String> extras = const [],
  ]) async {
    try {
      final r = await Process.run(executavel, [acao, base, '$fuso', ...extras]);
      final linha = (r.stdout as String)
          .split('\n')
          .where((s) => s.startsWith('DET_JSON:'))
          .lastOrNull;
      if (linha == null) {
        throw Exception('A ponte Rust não respondeu. ${r.stderr}');
      }
      final resposta = jsonDecode(linha.substring(9)) as Map<String, dynamic>;
      if (resposta['ok'] != true) {
        throw Exception(resposta['erro'] ?? 'Ação não concluída');
      }
      return resposta['dados'] as Map<String, dynamic>;
    } on ProcessException {
      throw Exception(
        'Executável Rust não encontrado. Rode cargo build --bin det-api.',
      );
    }
  }
}
