pipeline {
    options {
        timeout(time: 10, unit: 'MINUTES')
    }

    triggers {
        githubPush()
    }

    agent none
	stages {
        // stage('Build Windows') {
        //     agent {
        //         label 'windows'
        //     }
        //     steps {
        //         script {
        //             bat("build.bat")
        //         }
        //     }
        // }

        stage('Build Linux') {
            agent {
                label 'linux'
            }
            steps {
                script {
                    sh("chmod +x build.sh && ./build.sh")
                }
            }
        }

        stage('Deploy Linux') {
            agent {
                label 'linux'
            }
            when {
                branch 'master'
            }
            steps {
                withCredentials([string(credentialsId: 'allesctf-github-accesstoken', variable: 'GITHUB_TOKEN')]) {
                    script {
                        sh("chmod +x release.sh && ./release.sh")
                    }
                }
            }
        }
	}
}