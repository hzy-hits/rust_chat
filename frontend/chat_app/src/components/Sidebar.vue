<template>
    <div class="w-64 bg-gray-800 text-white flex flex-col h-screen p-4 text-sm">
      <div class="flex items-center justify-between mb-6">
        <div class="font-bold text-base truncate cursor-pointer" @click="toggleDropdown">
          <span>{{ workspaceName }}</span>
          <button class="text-gray-400 ml-1">&nbsp;▼</button>
        </div>
        <div 
          v-if="dropdownVisible" 
          class="absolute top-12 left-0 w-48 bg-gray-800 border border-gray-700 rounded-md shadow-lg z-10"
        >
          <ul class="py-1">
            <li @click="logout" class="px-4 py-2 hover:bg-gray-700 cursor-pointer">Logout</li>
            <!-- 你可以在此添加更多下拉菜单项 -->
          </ul>
        </div>
      </div>
  
      <!-- Channels 区域 -->
      <div class="mb-6">
        <div class="flex items-center justify-between mb-2">
          <h2 class="text-xs uppercase text-gray-400">Channels</h2>
          <button 
            @click="showChatModal(true)" 
            class="text-gray-400 text-xl hover:text-white"
          >
            +
          </button>
        </div>
        <ul>
          <li 
            v-for="channel in channels" 
            :key="channel.id" 
            @click="selectChannel(channel.id)"
            :class="[
              'px-2 py-1 rounded cursor-pointer', 
              { 'bg-blue-600': channel.id === activeChannelId }
            ]"
          >
            # {{ channel.name }}
          </li>
        </ul>
      </div>
  
      <!-- Direct Messages 区域 -->
      <div>
        <div class="flex items-center justify-between mb-2">
          <h2 class="text-xs uppercase text-gray-400">Direct Messages</h2>
          <button 
            @click="showChatModal(false)" 
            class="text-gray-400 text-xl hover:text-white"
          >
            +
          </button>
        </div>
        <ul>
          <li 
            v-for="channel in singleChannels" 
            :key="channel.id" 
            @click="selectChannel(channel.id)"
            :class="[
              'flex items-center px-2 py-1 rounded cursor-pointer', 
              { 'bg-blue-600': channel.id === activeChannelId }
            ]"
          >
            <img 
              :src="`https://ui-avatars.com/api/?name=${channel.recipient.username.replace(' ', '+')}`"
              class="w-6 h-6 rounded-full mr-2" 
              alt="Avatar" 
            />
            {{ channel.recipient.username }}
          </li>
        </ul>
      </div>
  
      <!-- 创建聊天模态框（群聊或私聊） -->
      <CreateChatModal 
        v-if="chatModalVisible"
        :visible="chatModalVisible"
        :is-group="isGroupChat"
        @update:visible="chatModalVisible = $event"
      />
    </div>
  </template>
  
  <script>
  import CreateChatModal from './CreateChatModal.vue'; // 请确保该组件存在并实现相应逻辑
  
  export default {
    components: {
      CreateChatModal,
    },
    data() {
      return {
        dropdownVisible: false,
        chatModalVisible: false, // 控制模态框显示
        isGroupChat: false,      // true:创建群聊  false:创建私聊
      };
    },
    computed: {
      workspaceName() {
        return this.$store.getters.getWorkspace.name || 'No Workspace';
      },
      channels() {
        return this.$store.getters.getChannels;
      },
      activeChannelId() {
        const channel = this.$store.state.activeChannel;
        return channel ? channel.id : null;
      },
      singleChannels() {
        return this.$store.getters.getSingChannels;
      },
    },
    methods: {
      toggleDropdown() {
        this.dropdownVisible = !this.dropdownVisible;
      },
      logout() {
        this.$store.dispatch('logout');
        this.$router.push('/login');
      },
      handleOutsideClick(event) {
        if (!this.$el.contains(event.target)) {
          this.dropdownVisible = false;
        }
      },
      selectChannel(channelId) {
        this.$store.dispatch('setActiveChannel', channelId);
      },
      showChatModal(isGroup) {
        this.isGroupChat = isGroup;
        this.chatModalVisible = true;
      },
    },
    mounted() {
      document.addEventListener('click', this.handleOutsideClick);
    },
    beforeDestroy() {
      document.removeEventListener('click', this.handleOutsideClick);
    },
  };
  </script>
  
  <style scoped>
  
  </style>
  