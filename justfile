curr_version := "breez_liquid-v" + `awk '/^version: /{print $2}' packages/breez_liquid/pubspec.yaml`
frb_bin := "flutter_rust_bridge_codegen generate"

export CARGO_TERM_COLOR := "always"

# generate bindings
gen: codegen && ffigen

codegen:
	{{frb_bin}}

ffigen:
	cd packages/flutter_breez_liquid/ && flutter pub run ffigen --config ffigen.yaml && cd ..

# builds the local library for testing
build *args:
	cargo build --manifest-path breez_liquid_wrapper/Cargo.toml {{args}}

build-apple *args:
	dart scripts/build_apple.dart {{args}}

build-android profile='release':
	bash scripts/build-android.sh {{profile}}

build-other *args:
	dart scripts/build_other.dart {{args}}

# (melos)
test: test-dart # test-flutter

# (melos)
test-dart: build
	melos run test-dart

# softlinks library archives from platform-build to their expected locations
link:
	-ln -sf $(pwd)/platform-build/breez_liquid_wrapper.xcframework.zip packages/flutter_breez_liquid/macos/Frameworks/{{curr_version}}.zip
	-ln -sf $(pwd)/platform-build/breez_liquid_wrapper.xcframework.zip packages/flutter_breez_liquid/ios/Frameworks/{{curr_version}}.zip
	-ln -sf $(pwd)/platform-build/other.tar.gz packages/flutter_breez_liquid/linux/{{curr_version}}.tar.gz
	-ln -sf $(pwd)/platform-build/other.tar.gz packages/flutter_breez_liquid/windows/{{curr_version}}.tar.gz
	-ln -sf $(pwd)/platform-build/android.tar.gz packages/flutter_breez_liquid/android/{{curr_version}}.tar.gz
	-ln -sf $(pwd)/breez_liquid_wrapper/include/breez_liquid_wrapper.h packages/flutter_breez_liquid/ios/Classes/breez_liquid_wrapper.h
	-ln -sf $(pwd)/breez_liquid_wrapper/include/breez_liquid_wrapper.h packages/flutter_breez_liquid/macos/Classes/breez_liquid_wrapper.h

# (melos)
test-flutter: build-apple build-android build-other
	melos run test-flutter

# (melos) use instead of flutter pub get
init *args:
	melos bootstrap {{args}}

# (melos) generate docs
docs:
	melos run docs

# (melos)
clean:
	melos clean

check:
	flutter analyze

# Open melos.yaml
melos:
	@$EDITOR melos.yaml
