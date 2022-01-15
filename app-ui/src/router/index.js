import { createRouter, createWebHistory } from 'vue-router';
import HomePage from '../views/HomePage.vue';
import ItemsPage from '../views/ItemsPage.vue';
import LocationsPage from '../views/LocationsPage.vue';
import TransactionsPage from '../views/TransactionsPage.vue';

const routes = [
    { path: '/', component: HomePage },
    { path: '/items', component: ItemsPage },
    { path: '/locations', component: LocationsPage },
    { path: '/transactions', component: TransactionsPage },
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

export default router;