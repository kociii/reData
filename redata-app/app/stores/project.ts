import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Project, CreateProjectRequest, UpdateProjectRequest } from '~/types'
import { projectsApi } from '~/utils/api'

export const useProjectStore = defineStore('project', () => {
  // State
  const projects = ref<Project[]>([])
  const currentProject = ref<Project | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const projectCount = computed(() => projects.value.length)
  const hasProjects = computed(() => projects.value.length > 0)

  // Actions
  async function fetchProjects() {
    loading.value = true
    error.value = null
    try {
      projects.value = await projectsApi.list()
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to fetch projects:', e)
    } finally {
      loading.value = false
    }
  }

  async function fetchProject(id: number) {
    loading.value = true
    error.value = null
    try {
      currentProject.value = await projectsApi.get(id)
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to fetch project:', e)
    } finally {
      loading.value = false
    }
  }

  async function createProject(data: CreateProjectRequest) {
    loading.value = true
    error.value = null
    try {
      const project = await projectsApi.create(data)
      projects.value.push(project)
      return project
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to create project:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function updateProject(id: number, data: UpdateProjectRequest) {
    loading.value = true
    error.value = null
    try {
      const project = await projectsApi.update(id, data)
      const index = projects.value.findIndex((p) => p.id === id)
      if (index !== -1) {
        projects.value[index] = project
      }
      if (currentProject.value?.id === id) {
        currentProject.value = project
      }
      return project
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to update project:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function deleteProject(id: number) {
    loading.value = true
    error.value = null
    try {
      await projectsApi.delete(id)
      projects.value = projects.value.filter((p) => p.id !== id)
      if (currentProject.value?.id === id) {
        currentProject.value = null
      }
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to delete project:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  function setCurrentProject(project: Project | null) {
    currentProject.value = project
  }

  function clearError() {
    error.value = null
  }

  return {
    // State
    projects,
    currentProject,
    loading,
    error,
    // Getters
    projectCount,
    hasProjects,
    // Actions
    fetchProjects,
    fetchProject,
    createProject,
    updateProject,
    deleteProject,
    setCurrentProject,
    clearError,
  }
})
