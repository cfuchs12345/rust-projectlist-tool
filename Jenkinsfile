pipeline {
    agent {label 'rust-slave'}

    stages {
        stage('Init') {
            steps {
                sh "rustup default stable"
            }
        }
        stage('Build') {
            steps {
                sh "cargo build --release"
            }
        }
        stage('Test') {
            steps {
                sh "cargo test"
            }
        }
        stage('Clippy') {
            steps {
                sh "cargo clippy --all"
            }
        }
        stage('Rustfmt') {
            steps {
                // The build will fail if rustfmt thinks any changes are
                // required.
                sh "cargo fmt --all"
            }
        }
        stage('Doc') {
            steps {
                sh "cargo doc"
                // We run a python `SimpleHTTPServer` against
                // /var/lib/jenkins/jobs/<repo>/branches/master/javadoc to
                // display our docs
                step([$class: 'JavadocArchiver',
                      javadocDir: 'target/doc',
                      keepAll: false])
            }
        }
    }
    post{
        always{
            emailext body: 'Build executed',
            recipientProviders: [developers(), requestor()],
            subject: 'jenkins build ${JOB_NAME}: ${BUILD_STATUS}',
            to: 'christopher@christopherfuchs.de'
        }
    }
}