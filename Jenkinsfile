pipeline {
    options {
        timeout(time: 10, unit: 'MINUTES')
    }

    triggers {
        githubPush()
    }

    agent none
	stages {
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

        // stage('Deploy') {
        //     when {
        //         environment name: 'DEPLOY', value: 'true'
        //     }
        //     steps {
        //         // sh label: '', returnStatus: true, script: '''cp jenkinsexample ~
        //         // cp test/testPro ~'''
        //     }
        // }
	}
}