// SPDX-FileCopyrightText: 2026 Nextcloud GmbH and Nextcloud contributors
// SPDX-License-Identifier: Apache-2.0

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.android.builtInKotlin)
    `maven-publish`
}

// Set by the release workflow
val releaseVersion = System.getenv("VERSION") ?: "0.0.0"

android {
    namespace = "com.nextcloud.talk.olm"
    compileSdk = 36

    defaultConfig {
        minSdk = 26
        consumerProguardFiles("consumer-rules.pro")
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    testOptions {
        // Build.VERSION.SDK_INT is 0 on the JVM, so the bindings fall back to the JNA cleaner
        unitTests.isReturnDefaultValues = true
    }

    publishing {
        singleVariant("release")
    }
}

// The notices ship inside the AAR, as in the XCFramework
android.sourceSets["main"].resources.srcDir(layout.buildDirectory.dir("notices"))

val copyNotices by tasks.registering(Copy::class) {
    from(rootDir.parentFile.resolve("THIRD_PARTY_LICENSES.md"))
    from(rootDir.parentFile.resolve("LICENSES/Apache-2.0.txt")) { rename { "LICENSE.txt" } }
    into(layout.buildDirectory.dir("notices/META-INF"))
}

tasks.named("preBuild") {
    dependsOn(copyNotices)
}

tasks.withType<Test>().configureEach {
    // JVM tests load the host library built by `cargo build --release` instead of the jniLibs
    systemProperty(
        "jna.library.path",
        System.getenv("JNA_LIBRARY_PATH") ?: rootDir.parentFile.resolve("target/release").path
    )
}

dependencies {
    api(libs.jna) { artifact { type = "aar" } }
    implementation(libs.annotation)
    testImplementation(libs.junit)
}

publishing {
    publications {
        create<MavenPublication>("release") {
            groupId = "com.github.nextcloud-releases"
            artifactId = "talk-clients-vodozemac"
            version = releaseVersion
            afterEvaluate { from(components["release"]) }
            pom {
                name = "talk-clients-vodozemac"
                description = "Olm bindings for the Nextcloud Talk clients, built on vodozemac"
                url = "https://github.com/nextcloud-releases/talk-clients-vodozemac"
                licenses {
                    license {
                        name = "Apache-2.0"
                        url = "https://www.apache.org/licenses/LICENSE-2.0"
                    }
                }
                scm {
                    url = "https://github.com/nextcloud-releases/talk-clients-vodozemac"
                }
            }
        }
    }
}
