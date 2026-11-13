import 'package:det_app/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('mostra as etapas do DET', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(home: Painel(carregarAoAbrir: false)),
    );
    expect(find.text('Gestor de Documentos de Testes'), findsOneWidget);
    expect(find.text('Criar subpastas'), findsOneWidget);
    expect(find.text('Gerar DETs em Word'), findsOneWidget);
  });
}
