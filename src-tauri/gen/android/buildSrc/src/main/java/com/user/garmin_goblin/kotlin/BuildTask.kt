import java.io.File
import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.TaskAction

open class BuildTask : DefaultTask() {
    @Input
    var rootDirRel: String? = null
    @Input
    var target: String? = null
    @Input
    var release: Boolean? = null

    @TaskAction
    fun assemble() {
        val ndk = System.getenv("NDK_HOME") ?: System.getenv("NDK") ?: "/home/user/Android/Sdk/ndk/30.0.14904198"
        val targetTriple = when (target) {
            "aarch64" -> "aarch64-linux-android"
            "armv7" -> "armv7-linux-androideabi"
            "i686" -> "i686-linux-android"
            "x86_64" -> "x86_64-linux-android"
            else -> "aarch64-linux-android"
        }
        val clangTarget = when (target) {
            "aarch64" -> "aarch64-linux-android24-clang"
            "armv7" -> "armv7a-linux-androideabi24-clang"
            "i686" -> "i686-linux-android24-clang"
            "x86_64" -> "x86_64-linux-android24-clang"
            else -> "aarch64-linux-android24-clang"
        }
        val clangCppTarget = when (target) {
            "aarch64" -> "aarch64-linux-android24-clang++"
            "armv7" -> "armv7a-linux-androideabi24-clang++"
            "i686" -> "i686-linux-android24-clang++"
            "x86_64" -> "x86_64-linux-android24-clang++"
            else -> "aarch64-linux-android24-clang++"
        }

        val llvmBin = File(ndk, "toolchains/llvm/prebuilt/linux-x86_64/bin")
        val targetEnvKey = targetTriple.uppercase().replace("-", "_")
        val targetEnvKeySmall = targetTriple.replace("-", "_")

        val releaseFlag = if (release == true) "--release" else ""
        val cmd = """
            export PATH=/home/user/.cargo/bin:${llvmBin.absolutePath}:${'$'}PATH
            export CC_$targetEnvKeySmall=${File(llvmBin, clangTarget).absolutePath}
            export CXX_$targetEnvKeySmall=${File(llvmBin, clangCppTarget).absolutePath}
            export AR_$targetEnvKeySmall=${File(llvmBin, "llvm-ar").absolutePath}
            export CARGO_TARGET_${targetEnvKey}_LINKER=${File(llvmBin, clangTarget).absolutePath}
            export CARGO_TARGET_${targetEnvKey}_RUSTFLAGS="-Clink-arg=-landroid -Clink-arg=-llog -Clink-arg=-lOpenSLES"
            cargo build --target $targetTriple $releaseFlag
        """.trimIndent()

        project.exec {
            workingDir(File("/home/user/Documents/garmin/garmin-goblin/src-tauri"))
            executable("bash")
            args("-c", cmd)
        }.assertNormalExitValue()
    }
}