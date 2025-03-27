import org.gradle.api.DefaultTask
import org.gradle.api.tasks.TaskAction
import java.io.File

abstract class CleanJniLibsTask : DefaultTask() {
	init {
		group = "cleanup"
		description = "Deletes all .so files in jniLibs directory"
	}

	@TaskAction
	fun cleanJniLibs() {
		val jniLibsDir = File(project.projectDir, "/src/main/jniLibs")
		if (jniLibsDir.exists() && jniLibsDir.isDirectory) {
			val deletedFiles = jniLibsDir.walkTopDown()
				.filter { it.isFile && it.extension == "so" }
				.map { file ->
					val deleted = file.delete()
					if (deleted) println("Deleted: ${file.path}")
					deleted
				}
				.count { it }
			println("Total .so files deleted: $deletedFiles")
		} else {
			println("jniLibs directory not found at: ${jniLibsDir.path}")
		}
	}
}
