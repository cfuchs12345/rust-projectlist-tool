pipeline {
    agent {label 'docker-slave'}

     parameters {
            string(
            name: "Branch_Name", 
            defaultValue: 'master', 
            description: '')
            string(
            name: "source_project",
            defaultValue: 'Rust/(Rust) Projektlist Tool',
            description: 'source project that generates the rust executable',
            )
             string(
            name: "artifact_file",
            defaultValue: 'target/rust_projectlisttool_webserver.zip',
            description: 'artifact name of the source project',
            )        
            string(
            name: "UBUNTU_VERSION", 
            defaultValue: 'focal', 
            description: 'version of ubuntu base image')
            string(
            name: "IMAGE_NAME", 
            defaultValue: 'docker-rust-projectlisttool', 
            description: 'name of the image')
   
        booleanParam(
           name: "PushImage", 
           defaultValue: false)
    }

    stages {
        stage('Clean') {
            steps {
                 sh "docker image rm ${params.IMAGE_NAME}:latest || true"
                 sh "docker image prune -a -f || true"
                 sh "rm projectlisttool || true"
                 sh "rm ${params.artifact_file} || true"
            }
        }
        stage('pull artifact') {
            steps {
                copyArtifacts projectName: "${params.source_project}", selector: lastSuccessful()

                unzip zipFile: "${params.artifact_file}"
            }
        }

        stage('Build image') {
            steps {
                script {
                echo "Bulding docker images"
                def buildArgs = """\
                --build-arg UBUNTU_VERSION=${params.UBUNTU_VERSION} \
                --build-arg HTTP_PORT=8080 \
                -f Dockerfile \
                --no-cache \
                ."""
                docker.build(
                   "${params.IMAGE_NAME}:$BUILD_NUMBER",
                   buildArgs)
                }
            }
        }
        stage('Tag image') {
            steps {
                script {
                    echo "Tagging docker image"
                    sh "docker tag ${params.IMAGE_NAME}:$BUILD_NUMBER ${params.IMAGE_NAME}:latest";
                }
            }
        }
    }
    post{
        always{
            emailext body: 'Build executed',
            recipientProviders: [developers(), requestor()],
            subject: 'jenkins build ${JOB_DESCRIPTION}: ${BUILD_STATUS}',
            to: 'christopher@christopherfuchs.de'
        }
    }
}