pipeline {
    agent any

    environment {

        // Variables de entorno para Nexus y AWS Fargate
           DOCKER_REGISTRY = "10.0.0.8:8082"  // Por ejemplo, si usas Nexus, la IP/dominio de N
           NEXUS_REPO      = "repositorio-nexus"
           DOCKER_IMAGE    = "api-paralela"
           DOCKER_TAG      = "latest"
        // Credenciales de Nexus (si se requieren)
            NEXUS_USER     = "admin"
            NEXUS_PASSPRASE = "Mari0203"

           // Parámetros para AWS
           AWS_REGION      = "us-east-1"  // Cambia según tu región
           ECS_CLUSTER     = "paralelacluster"  // Nombre de tu clúster en ECS
           ECS_SERVICE     = "paralela-service"     // Nombre del servicio en ECS
           TASK_FAMILY     = "paralelatask"  // Familia de la definición de tarea en ECS
           EXECUTION_ROLE  = "arn:aws:iam::831926602540:role/ecsTaskExecutionRole" // Rol de ejecució
    }

       stages {

           stage('Checkout') {
               steps {
                   git branch: 'develop', url: 'https://github.com/marielapichardo/api-paralelapc2.git'
               }
           }

           stage('Build Docker Image') {
               steps {
                   script {
                       // Construir la imagen Docker
                       bat "docker build -t ${DOCKER_IMAGE}:${DOCKER_TAG} ."
                       // Etiquetar la imagen para el registro
                       bat "docker tag ${DOCKER_IMAGE}:${DOCKER_TAG} ${DOCKER_REGISTRY}/${NEXUS_REPO}/${DOCKER_IMAGE}:${DOCKER_TAG}"
                   }
               }
           }

           stage('Login en Nexus') {
                steps {
                    bat "docker login ${DOCKER_REGISTRY} -u ${NEXUS_USER} -p ${NEXUS_PASSPRASE}"
                }
            }
            stage('Push en Nexus') {
                steps {
                    script {
                        bat "docker push ${DOCKER_REGISTRY}/${NEXUS_REPO}/${DOCKER_IMAGE}:${DOCKER_TAG}"
                    }
                }
            }

        }
       

           
       post {
           success {
               echo 'Pipeline completado exitosamente y el despliegue en AWS Fargate se ha iniciado.'
           }
           failure {
               echo 'El pipeline ha fallado.'
           }
       }
}
