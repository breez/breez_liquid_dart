/// Dart bindings for the Breez Liquid library.
library;

export 'src/wrapper/model.dart';
export 'src/wrapper/wallet.dart';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'src/frb_generated.dart';

typedef BreezLiquidWrapper = RustLibApi;
typedef BreezLiquidWrapperImpl = RustLibApiImpl;

Future<void> initialize({ExternalLibrary? dylib}) {
  return RustLib.init(externalLibrary: dylib);
}
