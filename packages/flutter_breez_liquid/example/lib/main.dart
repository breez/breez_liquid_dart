import 'package:flutter_breez_liquid/flutter_breez_liquid.dart';
import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';

void main() async {
  await initialize();
  runApp(const MyApp());
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  static const String mnemonic =
      "cute gallery debris flame service used expect poverty clarify window demise slim";

  Future<Wallet> _initializeWallet() async {
    final dataDir = await getApplicationDocumentsDirectory();
    return (await Wallet.init(
      mnemonic: mnemonic,
      dataDir: dataDir.path,
      network: Network.liquid,
    )) as Wallet;
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(
          title: const Text('Breez Liquid Native Packages'),
        ),
        body: Padding(
          padding: const EdgeInsets.all(10),
          child: FutureBuilder<Wallet>(
            future: _initializeWallet(),
            initialData: null,
            builder: (context, walletSnapshot) {
              if (walletSnapshot.hasError) {
                return Text('Error: ${walletSnapshot.error}');
              }

              if (!walletSnapshot.hasData) {
                return const Text('Loading...');
              }
              final wallet = walletSnapshot.data!;

              return FutureBuilder<WalletInfo>(
                future: wallet.getInfo(withScan: false),
                initialData: null,
                builder: (context, snapshot) {
                  if (snapshot.hasError) {
                    return Text('Error: ${snapshot.error}');
                  }

                  if (!snapshot.hasData) {
                    return const Text('Loading...');
                  }

                  if (snapshot.requireData.balanceSat.isNaN) {
                    return const Text('No balance.');
                  }
                  final walletInfo = snapshot.data!;

                  return Text("Balance: ${walletInfo.balanceSat}\npubKey: ${walletInfo.pubkey}");
                },
              );
            },
          ),
        ),
      ),
    );
  }
}
