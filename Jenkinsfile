pipeline {
    options {
        timeout(time: 10, unit: 'MINUTES')
    }

    triggers {
        githubPush()
    }

    agent none
	stages {
        stage("Build and deploy Linux/Windows"){
            parallel {
                stage('Windows') {
                    agent {
                        label 'windows'
                    }

                    stages {
                        stage('Build Windows') {
                            steps {
                                script {
                                    bat("build.bat")
                                }
                            }
                        }

                        stage('Deploy Windows') {
                            when {
                                branch 'master'
                            }
                            steps {
                                withCredentials([string(credentialsId: 'allesctf-github-accesstoken', variable: 'GITHUB_TOKEN')]) {
                                    script {
                                        bat("release.bat")
                                    }
                                }
                            }
                        }
                    }
                }

                stage('Linux') {
                    agent {
                        label 'linux'
                    }

                    stages {
                        stage('Build Linux') {
                            steps {
                                script {
                                    sh("chmod +x build.sh && ./build.sh")
                                }
                            }
                        }

                        stage('Deploy Linux') {
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
            }
        }
    }
}