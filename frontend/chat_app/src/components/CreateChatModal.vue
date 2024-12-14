<template>
    <div v-if="visible" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-lg p-6 w-96">
        <h2 class="text-xl font-bold mb-4 text-gray-800">{{ isGroup ? 'Create Group Chat' : 'Start Direct Message' }}</h2>
        
        <!-- 群聊名称输入（只有群聊需要名字）-->
        <div v-if="isGroup" class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-1">Group Name</label>
          <input
            v-model="chatName"
            type="text"
            class="w-full px-3 py-2 border border-gray-300 rounded-md"
            placeholder="Enter group name"
          />
        </div>
  
        <!-- 如果是群聊且有名字，可允许选择 public 或 private -->
        <div v-if="isGroup && chatName.trim().length > 0" class="mb-4 flex items-center">
          <input
            type="checkbox"
            id="publicChat"
            v-model="isPublic"
            class="mr-2"
          />
          <label for="publicChat" class="text-sm font-medium text-gray-700">Public Chat</label>
        </div>
  
        <!-- 用户列表 -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-1">Select Users</label>
          <div class="max-h-60 overflow-y-auto">
            <div
              v-for="user in users"
              :key="user.id"
              class="flex items-center p-2 hover:bg-gray-50 cursor-pointer"
              @click="toggleUser(user)"
            >
              <input
                type="checkbox"
                :checked="selectedUsers.includes(user.id)"
                class="mr-2"
              />
              <img
                :src="`https://ui-avatars.com/api/?name=${user.username.replace(' ', '+')}`"
                class="w-8 h-8 rounded-full mr-2"
              />
              <span class="text-gray-700">{{ user.username }}</span>
            </div>
          </div>
        </div>
  
        <!-- 按钮组 -->
        <div class="flex justify-end space-x-2">
          <button
            @click="close"
            class="px-4 py-2 text-sm text-gray-700 bg-gray-100 rounded-md hover:bg-gray-200"
          >
            Cancel
          </button>
          <button
            @click="createChat"
            :disabled="!isValid"
            :class="[
              'px-4 py-2 text-sm text-white rounded-md',
              isValid ? 'bg-blue-600 hover:bg-blue-700' : 'bg-blue-300'
            ]"
          >
            Create
          </button>
        </div>
      </div>
    </div>
  </template>
  
  <script>
  export default {
    props: {
      visible: Boolean,
      isGroup: {
        type: Boolean,
        default: false
      }
    },
  
    data() {
      return {
        chatName: '',
        selectedUsers: [],
        users: [],
        isPublic: false, // 新增字段，用于控制是否public
      };
    },
  
    computed: {
      isValid() {
        if (this.isGroup) {
          return this.chatName.trim() && this.selectedUsers.length > 0;
        }
        return this.selectedUsers.length === 1;
      }
    },
  
    methods: {
      async loadUsers() {
        try {
          this.users = await this.$store.dispatch('fetchWorkspaceUsers');
        } catch (error) {
          console.error('Failed to load users:', error);
        }
      },
  
      toggleUser(user) {
        const index = this.selectedUsers.indexOf(user.id);
        if (index === -1) {
          if (!this.isGroup && this.selectedUsers.length >= 1) {
            this.selectedUsers = [user.id];
          } else {
            this.selectedUsers.push(user.id);
          }
        } else {
          this.selectedUsers.splice(index, 1);
        }
      },
  
      async createChat() {
  try {
    // 基本验证
    if (!this.selectedUsers || this.selectedUsers.length === 0) {
      throw new Error('Please select at least one user');
    }

    // 始终包含 public 字段的请求体
    const payload = {
      members: this.selectedUsers,
      type: this.isGroup ? (this.isPublic ? 'publicChannel' : 'privateChannel') : 'single',
      public: this.isPublic || false,  // 确保始终有 public 字段
    };

    // 如果有聊天名称且不是私聊，添加名称
    if (this.isGroup && this.chatName && this.chatName.trim()) {
      payload.name = this.chatName.trim();
    }

    const chat = await this.$store.dispatch('createNewChat', payload);
    
    if (chat && chat.id) {
      this.$store.dispatch('setActiveChannel', chat.id);
      this.close();
    }
  } catch (error) {
    console.error('Failed to create chat:', error);
    this.$emit('error', error.message || 'Failed to create chat');
  }
},
  
      close() {
        this.chatName = '';
        this.selectedUsers = [];
        this.isPublic = false; // 重置public选项
        this.$emit('update:visible', false);
      }
    },
  
    watch: {
      visible(newVal) {
        if (newVal) {
          this.$store.dispatch('fetchWorkspaceUsers')
            .then(users => {
              this.users = users; 
            })
            .catch(err => console.error(err));
        }
      }
    },
  
    mounted() {
      if (this.visible) {
        this.loadUsers();
      }
    }
  };
  </script>
  